#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::unwrap_used)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]

use std::io::{self, BufRead, Write};

use chess_core::{board::Board, piece_move::PieceMove};

use crate::evaluation::evaluate;

pub mod evaluation;

const ENGINE_NAME: &str = "tmforshaw_engine";
const AUTHOR_NAME: &str = "tmforshaw";

const SEARCH_DEPTH: i32 = 4;

fn main() {
    respond_to_uci();
}

#[test]
fn board_move_test() {
    let mut board = Board::default();

    board.apply_move(
        PieceMove::from_algebraic("e2e4").expect("Could not turn algebraic into PieceMove"),
    );
    board.undo_move();

    assert!(board.positions == Board::default().positions);
}

/// # Panics
/// Line can't be found in ``stdin.lines()``
/// Best move can't be converted into algebraic
/// ``Stdout`` can't be written to
pub fn respond_to_uci() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    let mut board = None;

    for line in stdin.lock().lines() {
        let line = line.expect("Could not find line in stdin.lines()");
        let tokens: Vec<_> = line.split_whitespace().collect();
        if tokens.is_empty() {
            continue;
        }

        match tokens[0] {
            "uci" => {
                writeln!(stdout, "id name {ENGINE_NAME}").unwrap();
                writeln!(stdout, "id author {AUTHOR_NAME}").unwrap();
                writeln!(stdout, "uciok").unwrap();
            }
            "isready" => {
                writeln!(stdout, "readyok").unwrap();
            }
            "position" => {
                board = handle_position_uci(&tokens[1..]);
            }
            "go" => {
                let mut best_move = None;
                if let Some(board) = &mut board {
                    best_move = handle_go_uci(board);
                }

                match best_move {
                    Some(best_move) => writeln!(
                        stdout,
                        "bestmove {}",
                        best_move
                            .to_algebraic()
                            .expect("Could not convert best move into algebraic")
                    )
                    .expect("Could not write best move to stdin"),
                    None => todo!(),
                }
            }
            "quit" => {
                break;
            }
            _ => {
                // Unknown command
            }
        }

        stdout.flush().expect("Could not flush stdin");
    }
}

/// # Panics
/// If board can't be created from fen
/// If an unknown command is given instead of ``startpos`` or ``fen``
#[must_use]
pub fn handle_position_uci(tokens: &[&str]) -> Option<Board> {
    match tokens[0] {
        "fen" => {
            let fen_string = tokens[1];

            let mut board = Board::from_fen(fen_string).expect("Could not create Board from FEN");

            if let Some(&moves_keyword) = tokens.get(2) {
                if moves_keyword == "moves" {
                    handle_moves_uci(&mut board, &tokens[3..]);
                }
            }

            Some(board)
        }
        "startpos" => {
            let mut board = Board::default();

            if let Some(&moves_keyword) = tokens.get(1) {
                if moves_keyword == "moves" {
                    handle_moves_uci(&mut board, &tokens[2..]);
                }
            }

            Some(board)
        }
        _ => {
            // Should never happen
            unreachable!()
        }
    }
}

pub fn handle_moves_uci(board: &mut Board, moves: &[&str]) {
    let moves = moves
        .iter()
        .map(|&move_str| PieceMove::from_algebraic(move_str))
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    for piece_move in moves {
        board.apply_move(piece_move);
    }
}

pub fn handle_go_uci(board: &mut Board) -> Option<PieceMove> {
    let mut best_move = None;
    let mut best_score = i32::MIN;

    for piece_move in board.get_all_possible_moves(board.get_player()) {
        board.apply_move(piece_move);
        let score = alpha_beta(board, SEARCH_DEPTH - 1, i32::MIN, i32::MAX, false);
        board.undo_move();

        if score > best_score {
            best_score = score;
            best_move = Some(piece_move);
        }
    }

    best_move
}

pub fn alpha_beta(
    board: &mut Board,
    depth: i32,
    mut alpha: i32,
    mut beta: i32,
    maximising: bool,
) -> i32 {
    if depth <= 0 || board.has_game_ended().is_some() {
        return evaluate(board);
    }

    let mut best_score = if maximising { i32::MIN } else { i32::MAX };

    for piece_move in board.get_all_possible_moves(board.get_player()) {
        board.apply_move(piece_move);

        let score = alpha_beta(board, depth - 1, alpha, beta, !maximising);

        board.undo_move();

        if maximising {
            best_score = best_score.max(score);
            alpha = alpha.max(score);
            if alpha >= beta {
                break; // Beta cut‑off
            }
        } else {
            best_score = best_score.min(score);
            beta = beta.min(score);
            if beta <= alpha {
                break; // Alpha cut‑off
            }
        }
    }

    best_score
}
