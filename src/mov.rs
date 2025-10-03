use crate::{
    impl_from_str_for_parsable,
    parser::{self, ParseError, Parser, ParserExt},
    Color, ColoredPiece, Piece, Square,
};
use std::{
    array,
    fmt::{self, Display, Formatter},
    str::FromStr,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OpeningMove {
    pub color: Color,
    // From square 0 or square 63.
    pub pieces: [Piece; Self::SIZE],
}

impl OpeningMove {
    pub const SIZE: usize = 16;

    fn parser() -> impl Parser<Output = Self> {
        ColoredPiece::parser()
            .repeat(OpeningMove::SIZE..=OpeningMove::SIZE)
            .try_map(|colored_pieces| {
                let color = colored_pieces[0].color();
                if colored_pieces.iter().any(|p| p.color() != color) {
                    return Err(ParseError);
                }
                let mut pieces = array::from_fn(|i| colored_pieces[i].piece());
                match color {
                    Color::Red => {}
                    Color::Blue => {
                        pieces.reverse();
                    }
                }
                Ok(OpeningMove { color, pieces })
            })
    }
}

impl_from_str_for_parsable!(OpeningMove);

impl Display for OpeningMove {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let color = self.color;
        let mut pieces = self.pieces;
        match color {
            Color::Red => {}
            Color::Blue => {
                pieces.reverse();
            }
        }
        for piece in pieces {
            write!(f, "{}", piece.with_color(color))?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RegularMove {
    pub colored_piece: ColoredPiece,
    pub from: Option<Square>,
    pub captured: Option<Piece>,
    pub to: Square,
}

impl RegularMove {
    pub fn parser() -> impl Parser<Output = Self> {
        ColoredPiece::parser()
            .and_then(move |cpiece| {
                parser::exact(b"@")
                    .map(|_| (None, None)) // (from, captured)
                    .or(Square::parser().map(Some).and(
                        parser::exact(b"-").map(|_| None).or(parser::exact(b"x")
                            .ignore_then(ColoredPiece::parser())
                            .try_map(move |cpiece2| {
                                if cpiece2.color() != cpiece.color().opposite() {
                                    return Err(ParseError);
                                }
                                Ok(Some(cpiece2.piece()))
                            })),
                    ))
                    .map(move |(from, captured)| (cpiece, from, captured))
            })
            .and(Square::parser())
            .map(|((colored_piece, from, captured), to)| RegularMove {
                colored_piece,
                from,
                captured,
                to,
            })
    }
}

impl_from_str_for_parsable!(RegularMove);

impl Display for RegularMove {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.colored_piece)?;
        match (self.from, self.captured) {
            (None, None) => write!(f, "@")?,
            (None, Some(_)) => panic!("Drop capture"),
            (Some(from), None) => write!(f, "{from}-")?,
            (Some(from), Some(captured)) => {
                let captured_piece = captured.with_color(self.colored_piece.color().opposite());
                write!(f, "{from}x{captured_piece}")?;
            }
        }
        write!(f, "{}", self.to)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Move {
    Opening(OpeningMove),
    Regular(RegularMove),
}

impl Move {
    pub fn parser() -> impl Parser<Output = Self> {
        OpeningMove::parser()
            .map(Move::from)
            .or(RegularMove::parser().map(Move::from))
    }
}

impl_from_str_for_parsable!(Move);

impl From<OpeningMove> for Move {
    fn from(mov: OpeningMove) -> Self {
        Move::Opening(mov)
    }
}

impl From<RegularMove> for Move {
    fn from(mov: RegularMove) -> Self {
        Move::Regular(mov)
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Move::Opening(mov) => write!(f, "{mov}"),
            Move::Regular(mov) => write!(f, "{mov}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShortMove {
    Opening(OpeningMove),
    Slide {
        from: Square,
        to: Square,
    },
    Drop {
        colored_piece: ColoredPiece,
        to: Square,
    },
}

impl Display for ShortMove {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ShortMove::Opening(mov) => write!(f, "{mov}"),
            ShortMove::Slide { from, to } => write!(f, "{from}{to}"),
            ShortMove::Drop { colored_piece, to } => write!(f, "{colored_piece}{to}"),
        }
    }
}

impl From<Move> for ShortMove {
    fn from(mov: Move) -> Self {
        match mov {
            Move::Opening(mov) => ShortMove::Opening(mov),
            Move::Regular(mov) => match mov.from {
                None => ShortMove::Drop {
                    colored_piece: mov.colored_piece,
                    to: mov.to,
                },
                Some(from) => ShortMove::Slide { from, to: mov.to },
            },
        }
    }
}
