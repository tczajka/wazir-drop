use crate::{Bitboard, Color, Piece};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    to_move: Color,
    sides: [PositionSide; 2],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct PositionSide {
    piece_bitboards: [Bitboard; Piece::COUNT],
    num_captured: [u8; Piece::COUNT],
}
