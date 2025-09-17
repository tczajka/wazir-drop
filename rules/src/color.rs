use crate::ParseError;
use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Color {
    Red,
    Blue,
}

impl Color {
    pub const COUNT: usize = 2;

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Self::Red,
            1 => Self::Blue,
            _ => panic!("Invalid Color index"),
        }
    }

    pub fn opposite(self) -> Self {
        match self {
            Self::Red => Self::Blue,
            Self::Blue => Self::Red,
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Red => "red",
            Self::Blue => "blue",
        };
        write!(f, "{name}")
    }
}

impl FromStr for Color {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, ParseError> {
        match s {
            "red" => Ok(Self::Red),
            "blue" => Ok(Self::Blue),
            _ => Err(ParseError),
        }
    }
}
