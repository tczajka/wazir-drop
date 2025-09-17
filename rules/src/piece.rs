use crate::{square::Vector, Color, ParseError};
use std::{fmt::{self, Display, Formatter}, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Piece {
    Alfil,
    Dabbaba,
    Ferz,
    Knight,
    Wazir,
}

impl Piece {
    pub const COUNT: usize = 5;

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Self::Alfil,
            1 => Self::Dabbaba,
            2 => Self::Ferz,
            3 => Self::Knight,
            4 => Self::Wazir,
            _ => panic!("Invalid PieceType index"),
        }
    }

    pub fn initial_count(self) -> usize {
        match self {
            Self::Alfil => 8,
            Self::Dabbaba => 4,
            Self::Ferz => 2,
            Self::Knight => 1,
            Self::Wazir => 1,
        }
    }

    pub fn move_vectors(self) -> &'static [Vector] {
        static ALFIL: [Vector; 4] = [
            Vector::new(-2, -2),
            Vector::new(2, -2),
            Vector::new(-2, 2),
            Vector::new(2, 2),
        ];

        static DABBABA: [Vector; 4] = [
            Vector::new(0, -2),
            Vector::new(-2, 0),
            Vector::new(2, 0),
            Vector::new(0, 2),
        ];

        static FERZ: [Vector; 4] = [
            Vector::new(-1, -1),
            Vector::new(1, -1),
            Vector::new(-1, 1),
            Vector::new(1, 1),
        ];

        static KNIGHT: [Vector; 8] = [
            Vector::new(-1, -2),
            Vector::new(1, -2),
            Vector::new(-2, -1),
            Vector::new(2, -1),
            Vector::new(-2, 1),
            Vector::new(2, 1),
            Vector::new(-1, 2),
            Vector::new(1, 2),
        ];

        static WAZIR: [Vector; 4] = [
            Vector::new(0, -1),
            Vector::new(-1, 0),
            Vector::new(1, 0),
            Vector::new(0, 1),
        ];

        match self {
            Self::Alfil => &ALFIL,
            Self::Dabbaba => &DABBABA,
            Self::Ferz => &FERZ,
            Self::Knight => &KNIGHT,
            Self::Wazir => &WAZIR,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColoredPiece {
    pub color: Color,
    pub piece: Piece,
}

impl Display for ColoredPiece {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use Color::*;
        use Piece::*;
        
        let name = match self {
            ColoredPiece { color: Red, piece: Alfil } => "A",
            ColoredPiece { color: Red, piece: Dabbaba } => "D",
            ColoredPiece { color: Red, piece: Ferz } => "F",
            ColoredPiece { color: Red, piece: Knight } => "N",
            ColoredPiece { color: Red, piece: Wazir } => "W",
            ColoredPiece { color: Blue, piece: Alfil } => "a",
            ColoredPiece { color: Blue, piece: Dabbaba } => "d",
            ColoredPiece { color: Blue, piece: Ferz } => "f",
            ColoredPiece { color: Blue, piece: Knight } => "n",
            ColoredPiece { color: Blue, piece: Wazir } => "w",
        };
        write!(f, "{name}")
    }
}

impl FromStr for ColoredPiece {
    type Err = ParseError;

    #[rustfmt::skip]
    fn from_str(s: &str) -> Result<Self, ParseError> {
        use Color::*;
        use Piece::*;

        match s {
            "A" => Ok(Self { color: Red, piece: Alfil }),
            "D" => Ok(Self { color: Red, piece: Dabbaba }),
            "F" => Ok(Self { color: Red, piece: Ferz }),
            "N" => Ok(Self { color: Red, piece: Knight }),
            "W" => Ok(Self { color: Red, piece: Wazir }),
            "a" => Ok(Self { color: Blue, piece: Alfil }),
            "d" => Ok(Self { color: Blue, piece: Dabbaba }),
            "f" => Ok(Self { color: Blue, piece: Ferz }),
            "n" => Ok(Self { color: Blue, piece: Knight }),
            "w" => Ok(Self { color: Blue, piece: Wazir }),
            _ => Err(ParseError),
        }
    }
}
