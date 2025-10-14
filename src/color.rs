use crate::{
    impl_from_str_for_parsable,
    parser::{self, ParseError, Parser, ParserExt},
    unsafe_simple_enum, Bitboard,
};
use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

    pub fn parser() -> impl Parser<Output = Self> {
        parser::exact(b"red")
            .map(|_| Self::Red)
            .or(parser::exact(b"blue").map(|_| Self::Blue))
    }

    pub fn initial_squares(self) -> Bitboard {
        match self {
            Color::Red => Bitboard::from_bits(0xffff),
            Color::Blue => Bitboard::from_bits(0xffff << 48),
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

impl_from_str_for_parsable!(Color);
