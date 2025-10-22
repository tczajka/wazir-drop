use crate::{
    impl_from_str_for_parsable,
    parser::{self, ParseError, Parser, ParserExt},
    unsafe_simple_enum, Color, Direction,
};
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

    pub const fn directions(self) -> &'static [Direction] {
        const ALFIL: [Direction; 4] = [
            Direction::new(-2, -2),
            Direction::new(2, -2),
            Direction::new(-2, 2),
            Direction::new(2, 2),
        ];

        const DABBABA: [Direction; 4] = [
            Direction::new(0, -2),
            Direction::new(-2, 0),
            Direction::new(2, 0),
            Direction::new(0, 2),
        ];

        const FERZ: [Direction; 4] = [
            Direction::new(-1, -1),
            Direction::new(1, -1),
            Direction::new(-1, 1),
            Direction::new(1, 1),
        ];

        const KNIGHT: [Direction; 8] = [
            Direction::new(-1, -2),
            Direction::new(1, -2),
            Direction::new(-2, -1),
            Direction::new(2, -1),
            Direction::new(-2, 1),
            Direction::new(2, 1),
            Direction::new(-1, 2),
            Direction::new(1, 2),
        ];

        const WAZIR: [Direction; 4] = [
            Direction::new(0, -1),
            Direction::new(-1, 0),
            Direction::new(1, 0),
            Direction::new(0, 1),
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
