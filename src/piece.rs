use crate::{
    enum_map::{SimpleEnum, SimpleEnumExt},
    impl_from_str_for_parsable,
    parser::{self, ParseError, Parser, ParserExt},
    unsafe_simple_enum, Color, Vector,
};
use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

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
        ColoredPiece::from_index(self.index() * Color::COUNT + color.index())
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
#[repr(u8)]
pub enum ColoredPiece {
    RedAlfil,
    BlueAlfil,
    RedDabbaba,
    BlueDabbaba,
    RedFerz,
    BlueFerz,
    RedKnight,
    BlueKnight,
    RedWazir,
    BlueWazir,
}

unsafe_simple_enum!(ColoredPiece, 10);

impl ColoredPiece {
    pub fn parser() -> impl Parser<Output = Self> {
        parser::byte().try_map(|b| match b {
            b'A' => Ok(ColoredPiece::RedAlfil),
            b'a' => Ok(ColoredPiece::BlueAlfil),
            b'D' => Ok(ColoredPiece::RedDabbaba),
            b'd' => Ok(ColoredPiece::BlueDabbaba),
            b'F' => Ok(ColoredPiece::RedFerz),
            b'f' => Ok(ColoredPiece::BlueFerz),
            b'N' => Ok(ColoredPiece::RedKnight),
            b'n' => Ok(ColoredPiece::BlueKnight),
            b'W' => Ok(ColoredPiece::RedWazir),
            b'w' => Ok(ColoredPiece::BlueWazir),
            _ => Err(ParseError),
        })
    }

    pub fn color(self) -> Color {
        Color::from_index(self.index() % Color::COUNT)
    }

    pub fn piece(self) -> Piece {
        Piece::from_index(self.index() / Color::COUNT)
    }
}

impl Display for ColoredPiece {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let name = match self {
            ColoredPiece::RedAlfil    => "A",
            ColoredPiece::BlueAlfil   => "a",
            ColoredPiece::RedDabbaba  => "D",
            ColoredPiece::BlueDabbaba => "d",
            ColoredPiece::RedFerz     => "F",
            ColoredPiece::BlueFerz    => "f",
            ColoredPiece::RedKnight   => "N",
            ColoredPiece::BlueKnight  => "n",
            ColoredPiece::RedWazir    => "W",
            ColoredPiece::BlueWazir   => "w",
        };
        write!(f, "{name}")
    }
}

impl_from_str_for_parsable!(ColoredPiece);
