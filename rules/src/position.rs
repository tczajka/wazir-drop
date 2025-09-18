use crate::{
    mov::Move, Bitboard, Color, ColoredMove, ColoredOpeningMove, ColoredPiece, ColoredRegularMove,
    Coord, OpeningMove, ParseError, Piece, RegularMove, Square,
};
use std::{
    array,
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
    piece_bitboards: [Bitboard; Piece::COUNT],
    num_captured: [u8; Piece::COUNT],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    stage: Stage,
    to_move: Color,
    sides: [PositionSide; 2],
}

impl Position {
    pub fn stage(&self) -> Stage {
        self.stage
    }

    pub fn to_move(&self) -> Color {
        self.to_move
    }

    pub fn colored_piece(&self, square: Square) -> Option<ColoredPiece> {
        for color_index in 0..Color::COUNT {
            let side = &self.sides[color_index];
            for piece_index in 0..Piece::COUNT {
                if side.piece_bitboards[piece_index].contains(square) {
                    return Some(ColoredPiece {
                        color: Color::from_index(color_index),
                        piece: Piece::from_index(piece_index),
                    });
                }
            }
        }
        None
    }

    pub fn num_captured(&self, colored_piece: ColoredPiece) -> usize {
        self.sides[colored_piece.color.index()].num_captured[colored_piece.piece.index()].into()
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
                let (piece_name, to) = s.split_at_checked(1).ok_or(ParseError)?;
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
                let (from, to) = s.split_at_checked(2).ok_or(ParseError)?;
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
        for color_index in 0..Color::COUNT {
            for piece_index in 0..Piece::COUNT {
                let colored_piece = ColoredPiece {
                    color: Color::from_index(color_index),
                    piece: Piece::from_index(piece_index),
                };
                for _ in 0..self.num_captured(colored_piece) {
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
            sides: [PositionSide {
                piece_bitboards: [Bitboard::EMPTY; Piece::COUNT],
                num_captured: [0; Piece::COUNT],
            }; 2],
        };

        let mut remaining_pieces: [usize; Piece::COUNT] =
            array::from_fn(|i| Color::COUNT * Piece::from_index(i).initial_count());

        // Parse captured pieces.
        for color_index in 0..Color::COUNT {
            let color = Color::from_index(color_index);
            let side = &mut position.sides[color_index];
            let line = lines.next().ok_or(ParseError)?;
            for i in 0..line.len() {
                let piece_name = line.get(i..i + 1).ok_or(ParseError)?;
                let colored_piece = ColoredPiece::from_str(piece_name)?;
                if colored_piece.color != color {
                    return Err(ParseError);
                }
                let piece_index = colored_piece.piece.index();
                if remaining_pieces[piece_index] == 0 {
                    return Err(ParseError);
                }
                remaining_pieces[piece_index] -= 1;
                side.num_captured[piece_index] += 1;
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
                let piece_index = colored_piece.piece.index();
                if remaining_pieces[piece_index] == 0 {
                    return Err(ParseError);
                }
                remaining_pieces[piece_index] -= 1;
                position.sides[colored_piece.color.index()].piece_bitboards[piece_index]
                    .add(square);
            }
        }

        if lines.next().is_some() {
            return Err(ParseError);
        }

        if remaining_pieces.iter().any(|&n| n != 0) {
            return Err(ParseError);
        }

        // TODO: Check wazir count depending on stage.
        // TODO: Check opening positions are correct.

        Ok(position)
    }
}
