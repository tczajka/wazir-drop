use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

#[derive(Debug, Clone, Copy)]
pub struct ParseSuccess<'a, T> {
    pub value: T,
    pub remaining: &'a [u8],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ParseError;

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Parse error")
    }
}

impl Error for ParseError {}

pub type ParseResult<'a, T> = Result<ParseSuccess<'a, T>, ParseError>;

pub trait Parser: Sized {
    type Output;
    fn parse<'a>(&self, input: &'a [u8]) -> ParseResult<'a, Self::Output>;
}

pub trait ParserExt: Parser {
    fn parse_all(&self, input: &[u8]) -> Result<Self::Output, ParseError> {
        let success = self.parse(input)?;
        if success.remaining.is_empty() {
            Ok(success.value)
        } else {
            Err(ParseError)
        }
    }

    fn then<P: Parser>(self, p: P) -> impl Parser<Output = (Self::Output, P::Output)> {
        PairParser { p1: self, p2: p }
    }
}

impl<'a, P: Parser> ParserExt for P {}

pub struct Byte;

impl Parser for Byte {
    type Output = u8;

    fn parse<'a>(&self, input: &'a [u8]) -> ParseResult<'a, u8> {
        match input {
            [] => Err(ParseError),
            [head, tail @ ..] => Ok(ParseSuccess {
                value: *head,
                remaining: &tail,
            }),
        }
    }
}

pub struct ExactParser<'a> {
    s: &'a [u8],
}

impl<'a> Parser for ExactParser<'a> {
    type Output = ();

    fn parse<'b>(&self, input: &'b [u8]) -> ParseResult<'b, ()> {
        if input.starts_with(self.s) {
            Ok(ParseSuccess {
                value: (),
                remaining: &input[self.s.len()..],
            })
        } else {
            Err(ParseError)
        }
    }
}

pub fn exact<'a>(s: &'a [u8]) -> impl Parser<Output = ()> + 'a {
    ExactParser { s }
}

pub struct PairParser<P1: Parser, P2: Parser> {
    p1: P1,
    p2: P2,
}

impl<P1: Parser, P2: Parser> Parser for PairParser<P1, P2> {
    type Output = (P1::Output, P2::Output);

    fn parse<'a>(&self, input: &'a [u8]) -> ParseResult<'a, (P1::Output, P2::Output)> {
        let success1 = self.p1.parse(input)?;
        let success2 = self.p2.parse(success1.remaining)?;
        Ok(ParseSuccess {
            value: (success1.value, success2.value),
            remaining: success2.remaining,
        })
    }
}
