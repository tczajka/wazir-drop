use crate::{ParseError, BOARD_HEIGHT, BOARD_SIZE, BOARD_WIDTH};
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
    pub const fn from_row_column(row: usize, column: usize) -> Self {
        assert!(row < BOARD_HEIGHT && column < BOARD_WIDTH);
        Self::from_index(row * BOARD_WIDTH + column)
    }

    pub const fn from_index(index: usize) -> Self {
        assert!(index < BOARD_SIZE);
        unsafe { mem::transmute(index as u8) }
    }

    pub const fn index(self) -> usize {
        self as usize
    }

    pub const fn row(self) -> usize {
        self as usize / BOARD_WIDTH
    }

    pub const fn column(self) -> usize {
        self as usize % BOARD_WIDTH
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let row = char::from(b'a' + self.row() as u8);
        let column = char::from(b'1' + self.column() as u8);
        write!(f, "{row}{column}")
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
        let row = usize::from(bytes[0] - b'a');
        let column = usize::from(bytes[1] - b'1');
        Ok(Self::from_row_column(row, column))
    }
}
