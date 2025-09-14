use crate::{Bitboard, Square};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Color {
    Red,
    Blue,
}

impl Color {
    pub const NUM_COLORS: usize = 2;

    pub const fn index(self) -> usize {
        self as usize
    }

    pub const fn from_index(index: usize) -> Self {
        match index {
            0 => Self::Red,
            1 => Self::Blue,
            _ => panic!("Invalid Color index"),
        }
    }

    pub const fn opposite(self) -> Self {
        match self {
            Self::Red => Self::Blue,
            Self::Blue => Self::Red,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PieceType {
    Alfil,
    Dabbaba,
    Ferz,
    Knight,
    Wazir,
}

impl PieceType {
    pub const NUM_TYPES: usize = 5;

    pub const fn index(self) -> usize {
        self as usize
    }

    pub const fn from_index(index: usize) -> Self {
        match index {
            0 => Self::Alfil,
            1 => Self::Dabbaba,
            2 => Self::Ferz,
            3 => Self::Knight,
            4 => Self::Wazir,
            _ => panic!("Invalid PieceType index"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    to_move: Color,
    sides: [PositionSide; 2],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct PositionSide {
    piece_bitboards: [Bitboard; PieceType::NUM_TYPES],
    num_captured: [u8; PieceType::NUM_TYPES],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Move {
    pub piece_type: PieceType,
    pub captured: Option<PieceType>,
    pub from: Location,
    pub to: Square,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Location {
    Square(Square),
    Captured,
}
