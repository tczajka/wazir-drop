use crate::either::Either;
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    ops::RangeBounds,
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
    fn parse<'a>(self, input: &'a [u8]) -> ParseResult<'a, Self::Output>;
}

pub trait ParserExt: Parser {
    fn parse_all(self, input: &[u8]) -> Result<Self::Output, ParseError> {
        self.then_ignore(End)
            .parse(input)
            .map(|result| result.value)
    }

    fn then<P: Parser>(self, p: P) -> Pair<Self, P> {
        Pair { p1: self, p2: p }
    }

    fn or<P: Parser>(self, p: P) -> Or<Self, P> {
        Or { p1: self, p2: p }
    }

    fn map<T, F: Fn(Self::Output) -> T>(self, f: F) -> Map<Self, T, F> {
        Map { parser: self, f }
    }

    fn then_ignore<P: Parser>(self, p: P) -> impl Parser<Output = Self::Output> {
        self.then(p).map(|(a, _)| a)
    }

    fn ignore_then<P: Parser>(self, p: P) -> impl Parser<Output = P::Output> {
        self.then(p).map(|(_, b)| b)
    }

    // Note: This will greedily match too many elements and fail.
    fn repeat<R: RangeBounds<usize>>(self, range: R) -> Repeat<Self, R>
    where
        Self: Clone,
    {
        Repeat {
            parser: self,
            range,
        }
    }
}

impl<P: Parser> ParserExt for P {}

#[derive(Debug, Clone, Copy)]
pub struct End;

impl Parser for End {
    type Output = ();

    fn parse<'a>(self, input: &'a [u8]) -> ParseResult<'a, ()> {
        if input.is_empty() {
            Ok(ParseSuccess {
                value: (),
                remaining: input,
            })
        } else {
            Err(ParseError)
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Byte;

impl Parser for Byte {
    type Output = u8;

    fn parse<'a>(self, input: &'a [u8]) -> ParseResult<'a, u8> {
        match input {
            [] => Err(ParseError),
            [head, tail @ ..] => Ok(ParseSuccess {
                value: *head,
                remaining: tail,
            }),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Exact<'a> {
    s: &'a [u8],
}

impl<'a> Parser for Exact<'a> {
    type Output = ();

    fn parse<'b>(self, input: &'b [u8]) -> ParseResult<'b, ()> {
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

pub fn exact<'a>(s: &'a [u8]) -> Exact<'a> {
    Exact { s }
}

#[derive(Debug, Clone, Copy)]
pub struct Pair<P1: Parser, P2: Parser> {
    p1: P1,
    p2: P2,
}

impl<P1: Parser, P2: Parser> Parser for Pair<P1, P2> {
    type Output = (P1::Output, P2::Output);

    fn parse<'a>(self, input: &'a [u8]) -> ParseResult<'a, (P1::Output, P2::Output)> {
        let success1 = self.p1.parse(input)?;
        let success2 = self.p2.parse(success1.remaining)?;
        Ok(ParseSuccess {
            value: (success1.value, success2.value),
            remaining: success2.remaining,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Or<P1: Parser, P2: Parser> {
    p1: P1,
    p2: P2,
}

impl<P1: Parser, P2: Parser> Parser for Or<P1, P2> {
    type Output = Either<P1::Output, P2::Output>;

    fn parse<'a>(self, input: &'a [u8]) -> ParseResult<'a, Either<P1::Output, P2::Output>> {
        if let Ok(ParseSuccess { value, remaining }) = self.p1.parse(input) {
            Ok(ParseSuccess {
                value: Either::Left(value),
                remaining,
            })
        } else {
            let ParseSuccess { value, remaining } = self.p2.parse(input)?;
            Ok(ParseSuccess {
                value: Either::Right(value),
                remaining,
            })
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Map<P: Parser, T, F: Fn(P::Output) -> T> {
    parser: P,
    f: F,
}

impl<P: Parser, T, F: Fn(P::Output) -> T> Parser for Map<P, T, F> {
    type Output = T;

    fn parse<'a>(self, input: &'a [u8]) -> ParseResult<'a, T> {
        let success = self.parser.parse(input)?;
        Ok(ParseSuccess {
            value: (self.f)(success.value),
            remaining: success.remaining,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Repeat<P: Parser, R: RangeBounds<usize>> {
    parser: P,
    range: R,
}

impl<P: Parser, R: RangeBounds<usize>> Parser for Repeat<P, R>
where
    P: Clone,
{
    type Output = Vec<P::Output>;

    fn parse<'a>(self, input: &'a [u8]) -> ParseResult<'a, Vec<P::Output>> {
        let mut output = Vec::new();
        let mut remaining_input = input;
        let mut count = 0;
        while let Ok(ParseSuccess { value, remaining }) = self.parser.clone().parse(remaining_input)
        {
            output.push(value);
            remaining_input = remaining;
            count += 1;
        }
        if !self.range.contains(&count) {
            return Err(ParseError);
        }
        Ok(ParseSuccess {
            value: output,
            remaining: remaining_input,
        })
    }
}
