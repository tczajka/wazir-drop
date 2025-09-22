use crate::{
    enum_map::EnumMap, mov::Move, Bitboard, Color, ColoredMove, ColoredOpeningMove, ColoredPiece,
    ColoredRegularMove, Coord, OpeningMove, ParseError, Piece, RegularMove, SimpleEnum, Square,
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
                    return Some(ColoredPiece { color, piece });
                }
            }
        }
        None
    }

    pub fn num_captured(&self, colored_piece: ColoredPiece) -> usize {
        self.sides[colored_piece.color].num_captured[colored_piece.piece].into()
    }

    pub fn is_legal_regular_move(&self, mov: RegularMove) -> bool {
        if self.stage != Stage::Regular {
            return false;
        }
        match mov.from {
            None => {
                // Drop move.
                let dropped = mov.piece.with_color(self.to_move);
                self.num_captured(dropped) != 0 && self.colored_piece(mov.to).is_none()
            }
            Some(from) => {
                // Jump move.
                let moved = mov.piece.with_color(self.to_move);
                let captured = mov
                    .captured
                    .map(|piece| piece.with_color(self.to_move.opposite()));

                // TODO: Check move vector.
                self.colored_piece(from) == Some(moved) && self.colored_piece(mov.to) == captured
            }
        }
    }

    pub fn parse_opening_move(&self, s: &str) -> Result<OpeningMove, ParseError> {
        if self.stage != Stage::Opening {
            return Err(ParseError);
        }
        let mov = ColoredOpeningMove::from_str(s)?;
        if mov.color != self.to_move {
            return Err(ParseError);
        }
        Ok(mov.mov)
    }

    pub fn parse_regular_move(&self, s: &str) -> Result<RegularMove, ParseError> {
        let mov = match s.len() {
            3 => {
                // Drop move.
                if !s.is_char_boundary(1) {
                    return Err(ParseError);
                }
                let (piece_name, to) = s.split_at(1);
                let colored_piece = ColoredPiece::from_str(piece_name)?;
                if colored_piece.color != self.to_move {
                    return Err(ParseError);
                }
                let piece = colored_piece.piece;
                let to = Square::from_str(to)?;
                RegularMove {
                    piece,
                    captured: None,
                    from: None,
                    to,
                }
            }
            4 => {
                // Jump move.
                if !s.is_char_boundary(2) {
                    return Err(ParseError);
                }
                let (from, to) = s.split_at(2);
                let from = Square::from_str(from)?;
                let to = Square::from_str(to)?;

                let colored_piece = self.colored_piece(from).ok_or(ParseError)?;
                if colored_piece.color != self.to_move {
                    return Err(ParseError);
                }
                let piece = colored_piece.piece;

                let captured = match self.colored_piece(to) {
                    None => None,
                    Some(colored_captured) => {
                        if colored_captured.color == self.to_move {
                            return Err(ParseError);
                        }
                        Some(colored_captured.piece)
                    }
                };

                RegularMove {
                    piece,
                    captured,
                    from: Some(from),
                    to,
                }
            }
            _ => return Err(ParseError),
        };
        if !self.is_legal_regular_move(mov) {
            return Err(ParseError);
        }
        Ok(mov)
    }

    pub fn parse_move(&self, s: &str) -> Result<Move, ParseError> {
        match self.stage {
            Stage::Opening => {
                let mov = self.parse_opening_move(s)?;
                Ok(Move::Opening(mov))
            }
            Stage::Regular => {
                let mov = self.parse_regular_move(s)?;
                Ok(Move::Regular(mov))
            }
            Stage::End => Err(ParseError),
        }
    }

    pub fn colored_opening_move(&self, mov: OpeningMove) -> ColoredOpeningMove {
        mov.with_color(self.to_move)
    }

    pub fn colored_regular_move(&self, mov: RegularMove) -> ColoredRegularMove {
        mov.with_color(self.to_move)
    }

    pub fn colored_move(&self, mov: Move) -> ColoredMove {
        mov.with_color(self.to_move)
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.stage)?;
        writeln!(f, "{}", self.to_move)?;
        for (color, side) in self.sides.iter() {
            for (piece, &count) in side.num_captured.iter() {
                let colored_piece = ColoredPiece { color, piece };
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
                if colored_piece.color != color {
                    return Err(ParseError);
                }
                if remaining_pieces[colored_piece.piece] == 0 {
                    return Err(ParseError);
                }
                remaining_pieces[colored_piece.piece] -= 1;
                side.num_captured[colored_piece.piece] += 1;
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
                if remaining_pieces[colored_piece.piece] == 0 {
                    return Err(ParseError);
                }
                remaining_pieces[colored_piece.piece] -= 1;
                position.sides[colored_piece.color].piece_bitboards[colored_piece.piece]
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
