use crate::{enum_map::EnumMap, parser::ParseError, Color, ColoredPiece, Piece, Square};
use std::{
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
}

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

impl FromStr for OpeningMove {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, ParseError> {
        if s.len() != OpeningMove::SIZE {
            return Err(ParseError);
        }

        let mut color: Option<Color> = None;
        let mut pieces = [Piece::Alfil; OpeningMove::SIZE];
        let mut remaining: EnumMap<Piece, usize> = EnumMap::from_fn(Piece::initial_count);

        for i in 0..OpeningMove::SIZE {
            let piece_name = s.get(i..i + 1).ok_or(ParseError)?;
            let colored_piece = ColoredPiece::from_str(piece_name)?;
            match color {
                None => color = Some(colored_piece.color),
                Some(c) => {
                    if c != colored_piece.color {
                        return Err(ParseError);
                    }
                }
            }
            if remaining[colored_piece.piece] == 0 {
                return Err(ParseError);
            }
            remaining[colored_piece.piece] -= 1;
            pieces[i] = colored_piece.piece;
        }
        let color = color.unwrap();
        match color {
            Color::Red => {}
            Color::Blue => {
                pieces.reverse();
            }
        }
        Ok(OpeningMove { color, pieces })
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
