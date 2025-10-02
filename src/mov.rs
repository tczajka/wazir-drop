use crate::{
    impl_from_str_for_parsable,
    parser::{ParseError, Parser, ParserExt},
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
    pub color: Color,
    pub piece: Piece,
    pub captured: Option<Piece>,
    pub from: Option<Square>,
    pub to: Square,
}

impl Display for RegularMove {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.from {
            None => {
                write!(
                    f,
                    "{}",
                    ColoredPiece {
                        color: self.color,
                        piece: self.piece
                    }
                )?;
            }
            Some(from) => {
                write!(f, "{from}")?;
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

impl Display for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Move::Opening(mov) => write!(f, "{mov}"),
            Move::Regular(mov) => write!(f, "{mov}"),
        }
    }
}
