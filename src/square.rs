use crate::{
    enums::SimpleEnum,
    impl_from_str_for_parsable,
    parser::{self, ParseError, Parser, ParserExt},
    unsafe_simple_enum,
};
use std::{
    fmt::{self, Display, Formatter},
    mem,
    str::FromStr,
};

#[rustfmt::skip]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Square {
    A1, A2, A3, A4, A5, A6, A7, A8,
    B1, B2, B3, B4, B5, B6, B7, B8,
    C1, C2, C3, C4, C5, C6, C7, C8,
    D1, D2, D3, D4, D5, D6, D7, D8,
    E1, E2, E3, E4, E5, E6, E7, E8,
    F1, F2, F3, F4, F5, F6, F7, F8,
    G1, G2, G3, G4, G5, G6, G7, G8,
    H1, H2, H3, H4, H5, H6, H7, H8,
}

unsafe_simple_enum!(Square, 64);

impl Square {
    pub fn parser() -> impl Parser<Output = Self> {
        Coord::parser().map(|coord| coord.into())
    }
}

impl_from_str_for_parsable!(Square);

impl From<Coord> for Square {
    fn from(coord: Coord) -> Self {
        let index = coord.y * (Coord::WIDTH as u8) + coord.x;
        unsafe { mem::transmute(index) }
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let coord = Coord::from(*self);
        write!(f, "{coord}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Coord {
    x: u8,
    y: u8,
}

impl Coord {
    pub const WIDTH: usize = 8;
    pub const HEIGHT: usize = 8;

    pub fn new(x: usize, y: usize) -> Self {
        assert!(x < Self::WIDTH && y < Self::HEIGHT);

        Self {
            x: x as u8,
            y: y as u8,
        }
    }

    pub fn x(self) -> usize {
        self.x as usize
    }

    pub fn y(self) -> usize {
        self.y as usize
    }

    pub fn parser() -> impl Parser<Output = Self> {
        parser::byte()
            .try_map(|b| match b {
                b'a'..=b'h' => Ok(b - b'a'),
                _ => Err(ParseError),
            })
            .then(parser::byte().try_map(|b| match b {
                b'1'..=b'8' => Ok(b - b'1'),
                _ => Err(ParseError),
            }))
            .map(|(y, x)| Coord { x, y })
    }
}

impl From<Square> for Coord {
    fn from(square: Square) -> Self {
        let index = square.index();
        Self {
            x: (index % Coord::WIDTH) as u8,
            y: (index / Coord::WIDTH) as u8,
        }
    }
}

impl_from_str_for_parsable!(Coord);

impl Display for Coord {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let x = char::from(b'1' + self.x);
        let y = char::from(b'a' + self.y);
        write!(f, "{y}{x}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Vector {
    x: i8,
    y: i8,
}

impl Vector {
    pub const MAX_X: isize = Coord::WIDTH as isize - 1;
    pub const MAX_Y: isize = Coord::HEIGHT as isize - 1;

    pub const fn new(x: isize, y: isize) -> Self {
        assert!(x >= -Self::MAX_X && x <= Self::MAX_X && y >= -Self::MAX_Y && y <= Self::MAX_Y);
        Self {
            x: x as i8,
            y: y as i8,
        }
    }

    pub fn x(self) -> isize {
        self.x as isize
    }

    pub fn y(self) -> isize {
        self.y as isize
    }
}
