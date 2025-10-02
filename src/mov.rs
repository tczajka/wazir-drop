use crate::{
    either::Either,
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
                let color = colored_pieces[0].color;
                if colored_pieces.iter().any(|p| p.color != color) {
                    return Err(ParseError);
                }
                let mut pieces = array::from_fn(|i| colored_pieces[i].piece);
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
        match color {
            Color::Red => {
                for &piece in &self.pieces {
                    write!(f, "{}", ColoredPiece { color, piece })?;
                }
            }
            Color::Blue => {
                for &piece in self.pieces.iter().rev() {
                    write!(f, "{}", ColoredPiece { color, piece })?;
                }
            }
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
            .then(
                // (from, colored_captured)
                parser::exact(b"@")
                    .or(Square::parser().then(
                        parser::exact(b"-")
                            .or(parser::exact(b"x").ignore_then(ColoredPiece::parser()))
                            .map(|captured| match captured {
                                Either::Left(()) => None,
                                Either::Right(square) => Some(square),
                            }),
                    ))
                    .map(|from_captured| match from_captured {
                        Either::Left(()) => (None, None),
                        Either::Right((from, captured)) => (Some(from), captured),
                    }),
            )
            .then(Square::parser())
            .try_map(|((colored_piece, (from, colored_captured)), to)| {
                let captured = match colored_captured {
                    None => None,
                    Some(colored_captured) => {
                        if colored_captured.color != colored_piece.color.opposite() {
                            return Err(ParseError);
                        }
                        Some(colored_captured.piece)
                    }
                };
                Ok(RegularMove {
                    colored_piece,
                    from,
                    captured,
                    to,
                })
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
                let captured_piece = ColoredPiece {
                    color: self.colored_piece.color.opposite(),
                    piece: captured,
                };
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
            .or(RegularMove::parser())
            .map(|mov| match mov {
                Either::Left(mov) => Move::Opening(mov),
                Either::Right(mov) => Move::Regular(mov),
            })
    }
}

impl_from_str_for_parsable!(Move);

impl Display for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Move::Opening(mov) => write!(f, "{mov}"),
            Move::Regular(mov) => write!(f, "{mov}"),
        }
    }
}
