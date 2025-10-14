use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    ops::{Bound, RangeBounds},
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

impl<P: Parser> Parser for &P {
    type Output = P::Output;
    fn parse<'b>(&self, input: &'b [u8]) -> ParseResult<'b, Self::Output> {
        (*self).parse(input)
    }
}

pub trait ParserExt: Parser {
    fn parse_all(&self, input: &[u8]) -> Result<Self::Output, ParseError> {
        self.then_ignore(End)
            .parse(input)
            .map(|result| result.value)
    }

    fn and<P: Parser>(self, p: P) -> impl Parser<Output = (Self::Output, P::Output)> {
        And { p1: self, p2: p }
    }

    fn and_then<P: Parser, F: Fn(Self::Output) -> P>(
        self,
        f: F,
    ) -> impl Parser<Output = P::Output> {
        AndThen { p1: self, f }
    }

    fn or<P: Parser<Output = Self::Output>>(self, p: P) -> impl Parser<Output = Self::Output> {
        Or { p1: self, p2: p }
    }

    fn try_map<T, F>(self, f: F) -> impl Parser<Output = T>
    where
        F: Fn(Self::Output) -> Result<T, ParseError>,
    {
        TryMap { parser: self, f }
    }

    fn map<T, F: Fn(Self::Output) -> T>(self, f: F) -> impl Parser<Output = T> {
        self.try_map(move |x| Ok(f(x)))
    }

    fn then_ignore<P: Parser>(self, p: P) -> impl Parser<Output = Self::Output> {
        self.and(p).try_map(|(a, _)| Ok(a))
    }

    fn ignore_then<P: Parser>(self, p: P) -> impl Parser<Output = P::Output> {
        self.and(p).try_map(|(_, b)| Ok(b))
    }

    // Note: This will greedily match too many elements and fail.
    fn repeat<R: RangeBounds<usize>>(self, range: R) -> impl Parser<Output = Vec<Self::Output>> {
        let min_count = match range.start_bound() {
            Bound::Included(&x) => x,
            Bound::Excluded(&x) => x.checked_add(1).unwrap(),
            Bound::Unbounded => 0,
        };
        let max_count = match range.end_bound() {
            Bound::Included(&x) => x,
            Bound::Excluded(&x) => x.checked_sub(1).unwrap(),
            Bound::Unbounded => usize::MAX,
        };
        assert!(min_count <= max_count);
        Repeat {
            parser: self,
            min_count,
            max_count,
        }
    }
}

impl<P: Parser> ParserExt for P {}

#[derive(Debug, Clone, Copy)]
struct Empty;

pub fn empty() -> impl Parser<Output = ()> {
    Empty
}

impl Parser for Empty {
    type Output = ();

    fn parse<'a>(&self, input: &'a [u8]) -> ParseResult<'a, ()> {
        Ok(ParseSuccess {
            value: (),
            remaining: input,
        })
    }
}

#[derive(Debug, Clone, Copy)]
struct End;

pub fn end() -> impl Parser<Output = ()> {
    End
}

impl Parser for End {
    type Output = ();

    fn parse<'a>(&self, input: &'a [u8]) -> ParseResult<'a, ()> {
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
struct Byte;

impl Parser for Byte {
    type Output = u8;

    fn parse<'a>(&self, input: &'a [u8]) -> ParseResult<'a, u8> {
        match input {
            [] => Err(ParseError),
            [head, tail @ ..] => Ok(ParseSuccess {
                value: *head,
                remaining: tail,
            }),
        }
    }
}

pub fn byte() -> impl Parser<Output = u8> {
    Byte
}

#[derive(Debug, Clone, Copy)]
struct Exact<'a> {
    s: &'a [u8],
}

impl<'a> Parser for Exact<'a> {
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
    Exact { s }
}

#[derive(Debug, Clone, Copy)]
struct And<P1: Parser, P2: Parser> {
    p1: P1,
    p2: P2,
}

impl<P1: Parser, P2: Parser> Parser for And<P1, P2> {
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

#[derive(Debug, Clone, Copy)]
struct AndThen<P1: Parser, P2: Parser, F: Fn(P1::Output) -> P2> {
    p1: P1,
    f: F,
}

impl<P1: Parser, P2: Parser, F: Fn(P1::Output) -> P2> Parser for AndThen<P1, P2, F> {
    type Output = P2::Output;

    fn parse<'a>(&self, input: &'a [u8]) -> ParseResult<'a, P2::Output> {
        let success1 = self.p1.parse(input)?;
        let success2 = (self.f)(success1.value).parse(success1.remaining)?;
        Ok(success2)
    }
}

#[derive(Debug, Clone, Copy)]
struct Or<P1: Parser, P2: Parser<Output = P1::Output>> {
    p1: P1,
    p2: P2,
}

impl<P1: Parser, P2: Parser<Output = P1::Output>> Parser for Or<P1, P2> {
    type Output = P1::Output;

    fn parse<'a>(&self, input: &'a [u8]) -> ParseResult<'a, P1::Output> {
        if let Ok(ParseSuccess { value, remaining }) = self.p1.parse(input) {
            Ok(ParseSuccess { value, remaining })
        } else {
            let ParseSuccess { value, remaining } = self.p2.parse(input)?;
            Ok(ParseSuccess { value, remaining })
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct TryMap<P: Parser, F> {
    parser: P,
    f: F,
}

impl<P: Parser, T, F: Fn(P::Output) -> Result<T, ParseError>> Parser for TryMap<P, F> {
    type Output = T;

    fn parse<'a>(&self, input: &'a [u8]) -> ParseResult<'a, T> {
        let success = self.parser.parse(input)?;
        let value = (self.f)(success.value)?;
        Ok(ParseSuccess {
            value,
            remaining: success.remaining,
        })
    }
}

#[derive(Debug, Clone, Copy)]
struct Repeat<P: Parser> {
    parser: P,
    min_count: usize,
    max_count: usize,
}

impl<P: Parser> Parser for Repeat<P> {
    type Output = Vec<P::Output>;

    fn parse<'a>(&self, input: &'a [u8]) -> ParseResult<'a, Vec<P::Output>> {
        let mut output = Vec::new();
        let mut remaining_input = input;
        let mut count = 0;
        while count < self.max_count {
            let Ok(ParseSuccess { value, remaining }) = self.parser.parse(remaining_input) else {
                break;
            };
            output.push(value);
            remaining_input = remaining;
            count += 1;
        }
        if count < self.min_count {
            return Err(ParseError);
        }
        Ok(ParseSuccess {
            value: output,
            remaining: remaining_input,
        })
    }
}

#[macro_export]
macro_rules! impl_from_str_for_parsable {
    ($type:ty) => {
        impl FromStr for $type {
            type Err = ParseError;

            fn from_str(s: &str) -> Result<Self, ParseError> {
                Self::parser().parse_all(s.as_bytes())
            }
        }
    };
}
