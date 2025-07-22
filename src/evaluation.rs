use chess_core::{
    board::{Board, Player},
    piece::Piece,
};

#[must_use]
pub const fn piece_to_value(piece: Piece) -> i32 {
    match piece {
        Piece::WPawn | Piece::BPawn => 100,
        Piece::WKnight | Piece::BKnight => 320,
        Piece::WBishop | Piece::BBishop => 330,
        Piece::WRook | Piece::BRook => 500,
        Piece::WQueen | Piece::BQueen => 900,
        _ => 0,
    }
}

#[must_use]
pub fn material(board: &Board, player: Player) -> i32 {
    board
        .get_all_player_pieces(player)
        .into_iter()
        .map(|(piece, _)| piece)
        .fold(0, |acc, piece| acc + piece_to_value(piece))
}

#[must_use]
pub fn evaluate(board: &Board) -> i32 {
    material(board, Player::White) - material(board, Player::Black)
}
