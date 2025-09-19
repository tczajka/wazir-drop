use crate::{unsafe_simple_enum, ParseError};
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

unsafe_simple_enum!(Color, 2);

impl Color {
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
