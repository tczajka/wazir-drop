use crate::{enum_map::EnumMap, Color, ColoredPiece, Enum, ParseError, Piece, Square};
use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OpeningMove {
    // From square 0 or square 63.
    pub pieces: [Piece; Self::SIZE],
}

impl OpeningMove {
    pub const SIZE: usize = 16;

    pub fn with_color(self, color: Color) -> ColoredOpeningMove {
        ColoredOpeningMove { color, mov: self }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColoredOpeningMove {
    pub color: Color,
    pub mov: OpeningMove,
}

impl Display for ColoredOpeningMove {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let color = self.color;
        match color {
            Color::Red => {
                for &piece in &self.mov.pieces {
                    write!(f, "{}", ColoredPiece { color, piece })?;
                }
            }
            Color::Blue => {
                for &piece in self.mov.pieces.iter().rev() {
                    write!(f, "{}", ColoredPiece { color, piece })?;
                }
            }
        }
        Ok(())
    }
}

impl FromStr for ColoredOpeningMove {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, ParseError> {
        if s.len() != OpeningMove::SIZE {
            return Err(ParseError);
        }

        let mut color: Option<Color> = None;
        let mut pieces = [Piece::Alfil; OpeningMove::SIZE];
        let mut remaining: EnumMap<Piece, usize> = EnumMap::from_array([0; Piece::COUNT]);
        for (piece, r) in remaining.iter_mut() {
            *r = piece.initial_count();
        }

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
        Ok(ColoredOpeningMove {
            color,
            mov: OpeningMove { pieces },
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RegularMove {
    pub piece: Piece,
    pub captured: Option<Piece>,
    pub from: Option<Square>,
    pub to: Square,
}

impl RegularMove {
    pub fn with_color(self, color: Color) -> ColoredRegularMove {
        ColoredRegularMove { color, mov: self }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColoredRegularMove {
    pub color: Color,
    pub mov: RegularMove,
}

impl Display for ColoredRegularMove {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.mov.from {
            None => {
                write!(
                    f,
                    "{}",
                    ColoredPiece {
                        color: self.color,
                        piece: self.mov.piece
                    }
                )?;
            }
            Some(from) => {
                write!(f, "{from}")?;
            }
        }
        write!(f, "{}", self.mov.to)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Move {
    Opening(OpeningMove),
    Regular(RegularMove),
}

impl Move {
    pub fn with_color(self, color: Color) -> ColoredMove {
        ColoredMove { color, mov: self }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColoredMove {
    pub color: Color,
    pub mov: Move,
}

impl Display for ColoredMove {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.mov {
            Move::Opening(mov) => write!(
                f,
                "{}",
                ColoredOpeningMove {
                    color: self.color,
                    mov
                }
            ),
            Move::Regular(mov) => write!(
                f,
                "{}",
                ColoredRegularMove {
                    color: self.color,
                    mov
                }
            ),
        }
    }
}
