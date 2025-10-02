use crate::{parser::{self, ParseError, Parser, ParserExt, impl_from_str_for_parsable}, enum_map::unsafe_simple_enum, Color, Vector};
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

unsafe_simple_enum!(Piece, 5);

impl Piece {
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

    pub fn with_color(self, color: Color) -> ColoredPiece {
        ColoredPiece { color, piece: self }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PieceNonWazir {
    Alfil,
    Dabbaba,
    Ferz,
    Knight,
}

unsafe_simple_enum!(PieceNonWazir, 4);

impl From<PieceNonWazir> for Piece {
    fn from(piece: PieceNonWazir) -> Self {
        match piece {
            PieceNonWazir::Alfil => Self::Alfil,
            PieceNonWazir::Dabbaba => Self::Dabbaba,
            PieceNonWazir::Ferz => Self::Ferz,
            PieceNonWazir::Knight => Self::Knight,
        }
    }
}

impl TryFrom<Piece> for PieceNonWazir {
    type Error = ();

    fn try_from(piece: Piece) -> Result<Self, Self::Error> {
        match piece {
            Piece::Alfil => Ok(PieceNonWazir::Alfil),
            Piece::Dabbaba => Ok(PieceNonWazir::Dabbaba),
            Piece::Ferz => Ok(PieceNonWazir::Ferz),
            Piece::Knight => Ok(PieceNonWazir::Knight),
            Piece::Wazir => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColoredPiece {
    pub color: Color,
    pub piece: Piece,
}

impl ColoredPiece {
    pub fn parser() -> impl Parser<Output = Self> {
        use Color::*;
        use Piece::*;

        parser::byte().try_map(|b| match b {
            b'A' => Ok(Self { color: Red, piece: Alfil }),
            b'D' => Ok(Self { color: Red, piece: Dabbaba }),
            b'F' => Ok(Self { color: Red, piece: Ferz }),
            b'N' => Ok(Self { color: Red, piece: Knight }),
            b'W' => Ok(Self { color: Red, piece: Wazir }),
            b'a' => Ok(Self { color: Blue, piece: Alfil }),
            b'd' => Ok(Self { color: Blue, piece: Dabbaba }),
            b'f' => Ok(Self { color: Blue, piece: Ferz }),
            b'n' => Ok(Self { color: Blue, piece: Knight }),
            b'w' => Ok(Self { color: Blue, piece: Wazir }),
            _ => Err(ParseError),
        })
    }
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

impl_from_str_for_parsable!(ColoredPiece);
