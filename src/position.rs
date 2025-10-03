use crate::{
    enum_map::EnumMap, parser::ParseError, Bitboard, Color, ColoredPiece, Coord, Piece, Square,
};
use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stage {
    Opening,
    Regular,
    End,
}

impl Display for Stage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Stage::Opening => write!(f, "opening"),
            Stage::Regular => write!(f, "regular"),
            Stage::End => write!(f, "end"),
        }
    }
}

impl FromStr for Stage {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, ParseError> {
        match s {
            "opening" => Ok(Stage::Opening),
            "regular" => Ok(Stage::Regular),
            "end" => Ok(Stage::End),
            _ => Err(ParseError),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct PositionSide {
    piece_bitboards: EnumMap<Piece, Bitboard>,
    num_captured: EnumMap<Piece, u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    stage: Stage,
    to_move: Color,
    // TODO: square: piece, attack_count
    sides: EnumMap<Color, PositionSide>,
}

impl Position {
    pub fn stage(&self) -> Stage {
        self.stage
    }

    pub fn to_move(&self) -> Color {
        self.to_move
    }

    pub fn colored_piece(&self, square: Square) -> Option<ColoredPiece> {
        for (color, side) in self.sides.iter() {
            for (piece, bitboard) in side.piece_bitboards.iter() {
                if bitboard.contains(square) {
                    return Some(piece.with_color(color));
                }
            }
        }
        None
    }

    pub fn num_captured(&self, colored_piece: ColoredPiece) -> usize {
        self.sides[colored_piece.color()].num_captured[colored_piece.piece()].into()
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.stage)?;
        writeln!(f, "{}", self.to_move)?;
        for (color, side) in self.sides.iter() {
            for (piece, &count) in side.num_captured.iter() {
                let colored_piece = piece.with_color(color);
                for _ in 0..count {
                    write!(f, "{colored_piece}")?;
                }
            }
            writeln!(f)?;
        }
        for y in 0..Coord::HEIGHT {
            for x in 0..Coord::WIDTH {
                let square = Coord::new(x, y).into();
                match self.colored_piece(square) {
                    None => write!(f, ".")?,
                    Some(colored_piece) => write!(f, "{colored_piece}")?,
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl FromStr for Position {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, ParseError> {
        let mut lines = s.lines();

        // Parse stage.
        let stage = Stage::from_str(lines.next().ok_or(ParseError)?)?;

        // Parse to_move.
        let to_move = Color::from_str(lines.next().ok_or(ParseError)?)?;

        let mut position = Position {
            stage,
            to_move,
            sides: EnumMap::from_fn(|_| PositionSide {
                piece_bitboards: EnumMap::from_fn(|_| Bitboard::EMPTY),
                num_captured: EnumMap::from_fn(|_| 0),
            }),
        };

        // TODO: In opening, piece counts are different.
        let mut remaining_pieces: EnumMap<Piece, usize> = EnumMap::from_fn(|_| 0);
        for (piece, r) in remaining_pieces.iter_mut() {
            *r = 2 * piece.initial_count();
        }

        // Parse captured pieces.
        for (color, side) in position.sides.iter_mut() {
            let line = lines.next().ok_or(ParseError)?;
            for i in 0..line.len() {
                let piece_name = line.get(i..i + 1).ok_or(ParseError)?;
                let colored_piece = ColoredPiece::from_str(piece_name)?;
                if colored_piece.color() != color {
                    return Err(ParseError);
                }
                if remaining_pieces[colored_piece.piece()] == 0 {
                    return Err(ParseError);
                }
                remaining_pieces[colored_piece.piece()] -= 1;
                side.num_captured[colored_piece.piece()] += 1;
            }
        }

        // Parse board.
        for y in 0..Coord::HEIGHT {
            let line = lines.next().ok_or(ParseError)?;
            if line.len() != Coord::WIDTH {
                return Err(ParseError);
            }
            for x in 0..Coord::WIDTH {
                let square = Coord::new(x, y).into();
                let piece_name = line.get(x..x + 1).ok_or(ParseError)?;
                if piece_name == "." {
                    continue;
                }
                let colored_piece = ColoredPiece::from_str(piece_name)?;
                if remaining_pieces[colored_piece.piece()] == 0 {
                    return Err(ParseError);
                }
                remaining_pieces[colored_piece.piece()] -= 1;
                position.sides[colored_piece.color()].piece_bitboards[colored_piece.piece()]
                    .add(square);
            }
        }

        if lines.next().is_some() {
            return Err(ParseError);
        }

        if remaining_pieces.iter().any(|(_, &r)| r != 0) {
            return Err(ParseError);
        }

        // TODO: Check wazir count depending on stage.
        // TODO: Check opening positions are correct.

        Ok(position)
    }
}
