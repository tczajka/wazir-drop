use crate::ParseError;
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

impl Square {
    pub const COUNT: usize = Coord::HEIGHT * Coord::WIDTH;

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn from_index(index: usize) -> Self {
        assert!(index < Self::COUNT);
        unsafe { Self::unsafe_from_index(index) }
    }

    pub unsafe fn unsafe_from_index(index: usize) -> Self {
        unsafe { mem::transmute(index as u8) }
    }
}

impl From<Coord> for Square {
    fn from(coord: Coord) -> Self {
        unsafe { Self::unsafe_from_index(coord.y() * Coord::WIDTH + coord.x()) }
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let coord = Coord::from(*self);
        let x = char::from(b'1' + coord.x() as u8);
        let y = char::from(b'a' + coord.y() as u8);
        write!(f, "{y}{x}")
    }
}

impl FromStr for Square {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, ParseError> {
        let bytes = s.as_bytes();
        if bytes.len() != 2
            || !(b'a'..=b'h').contains(&bytes[0])
            || !(b'1'..=b'8').contains(&bytes[1])
        {
            return Err(ParseError);
        }
        let y = usize::from(bytes[0] - b'a');
        let x = usize::from(bytes[1] - b'1');
        Ok(Coord::new(x, y).into())
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
