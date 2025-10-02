use crate::{
    either::Either,
    impl_from_str_for_parsable,
    parser::{self, ParseError, Parser, ParserExt},
    unsafe_simple_enum,
};
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

    pub fn parser() -> impl Parser<Output = Self> {
        parser::exact(b"red")
            .or(parser::exact(b"blue"))
            .map(|result| match result {
                Either::Left(_) => Self::Red,
                Either::Right(_) => Self::Blue,
            })
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
