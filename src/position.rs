use crate::{
    enums::SimpleEnumExt,
    error::Invalid,
    impl_from_str_for_parsable, movegen,
    parser::{self, ParseError, Parser, ParserExt},
    unsafe_simple_enum, Bitboard, Board, Captured, Color, ColoredPiece, InvalidMove, Move, Piece,
    RegularMove, SetupMove, Square,
};
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Stage {
    Setup,
    Regular,
    End,
}

unsafe_simple_enum!(Stage, 3);

impl Stage {
    fn parser() -> impl Parser<Output = Self> {
        parser::exact(b"setup")
            .map(|_| Stage::Setup)
            .or(parser::exact(b"regular").map(|_| Stage::Regular))
            .or(parser::exact(b"end").map(|_| Stage::End))
    }
}

impl_from_str_for_parsable!(Stage);

impl Display for Stage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Stage::Setup => write!(f, "setup"),
            Stage::Regular => write!(f, "regular"),
            Stage::End => write!(f, "end"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Position {
    stage: Stage,
    to_move: Color,
    board: Board,
    captured: Captured,
}

impl Position {
    pub fn initial() -> Self {
        Self {
            stage: Stage::Setup,
            to_move: Color::Red,
            board: Board::empty(),
            captured: Captured::new(),
        }
    }

    pub fn stage(&self) -> Stage {
        self.stage
    }

    pub fn to_move(&self) -> Color {
        self.to_move
    }

    pub fn square(&self, square: Square) -> Option<ColoredPiece> {
        self.board.square(square)
    }

    pub fn occupied_by(&self, color: Color) -> Bitboard {
        self.board.occupied_by(color)
    }

    pub fn empty_squares(&self) -> Bitboard {
        self.board.empty_squares()
    }

    pub fn piece_map(&self, cpiece: ColoredPiece) -> Bitboard {
        self.board.piece_map(cpiece)
    }

    pub fn num_captured(&self, cpiece: ColoredPiece) -> usize {
        self.captured.get(cpiece)
    }

    pub fn parser() -> impl Parser<Output = Self> {
        Stage::parser()
            .then_ignore(parser::endl())
            .and(Color::parser())
            .then_ignore(parser::endl())
            .and(Captured::parser())
            .then_ignore(parser::endl())
            .and(Board::parser())
            .try_map(|(((stage, to_move), captured), board)| {
                Self::from_parts(stage, to_move, board, captured).map_err(|_| ParseError)
            })
    }

    fn from_parts(
        stage: Stage,
        to_move: Color,
        board: Board,
        captured: Captured,
    ) -> Result<Position, Invalid> {
        // Verify total piece count.
        if stage != Stage::Setup {
            for piece in Piece::all() {
                let mut count = 0;
                for color in Color::all() {
                    let cpiece = piece.with_color(color);
                    count += board.piece_map(cpiece).count();
                    count += captured.get(cpiece);
                }
                if count != Color::COUNT * piece.initial_count() {
                    return Err(Invalid);
                }
            }
        }

        match stage {
            Stage::Setup => {
                // Verify correct pieces placed in the right squares and nothing captured.
                for cpiece in ColoredPiece::all() {
                    let want = if cpiece.color() < to_move {
                        cpiece.piece().initial_count()
                    } else {
                        0
                    };
                    let squares = board.piece_map(cpiece);
                    if squares.count() != want
                        || !squares.is_subset_of(cpiece.color().initial_squares())
                        || captured.get(cpiece) != 0
                    {
                        return Err(Invalid);
                    }
                }
            }
            Stage::Regular => {
                // Verify one wazir per color on the board.
                for color in Color::all() {
                    if board.piece_map(Piece::Wazir.with_color(color)).count() != 1 {
                        return Err(Invalid);
                    }
                }
            }
            Stage::End => {
                // Verify opposite wazir on the board and one captured.
                let wazir_opp = Piece::Wazir.with_color(to_move.opposite());
                if board.piece_map(wazir_opp).count() != 1 || captured.get(wazir_opp) != 1 {
                    return Err(Invalid);
                }
            }
        }
        Ok(Position {
            stage,
            to_move,
            board,
            captured,
        })
    }

    pub fn make_move(&self, mov: Move) -> Result<Position, InvalidMove> {
        match mov {
            Move::Setup(mov) => self.make_setup_move(mov),
            Move::Regular(mov) => self.make_regular_move(mov),
        }
    }

    pub fn make_setup_move(&self, mov: SetupMove) -> Result<Position, InvalidMove> {
        let me = self.to_move();
        let opp = me.opposite();
        if self.stage != Stage::Setup || mov.color != me {
            return Err(InvalidMove);
        }
        mov.validate_pieces()?;
        let mut new_position = *self;
        for (i, &piece) in mov.pieces.iter().enumerate() {
            let square = Square::from_index(i).pov(mov.color);
            new_position
                .board
                .place_piece(square, piece.with_color(me))
                .unwrap();
        }
        new_position.to_move = opp;
        if opp == Color::Red {
            new_position.stage = Stage::Regular;
        }
        Ok(new_position)
    }

    pub fn make_regular_move(&self, mov: RegularMove) -> Result<Position, InvalidMove> {
        let me = self.to_move;
        let opp = me.opposite();
        if self.stage != Stage::Regular || mov.colored_piece.color() != me {
            return Err(InvalidMove);
        }
        let mut new_position = *self;
        match mov.from {
            None => {
                new_position
                    .captured
                    .remove(mov.colored_piece)
                    .map_err(|_| InvalidMove)?;
            }
            Some(from) => {
                movegen::validate_from_to(mov.colored_piece.piece(), from, mov.to)?;
                new_position
                    .board
                    .remove_piece(from, mov.colored_piece)
                    .map_err(|_| InvalidMove)?;
            }
        }
        if let Some(captured) = mov.captured {
            new_position
                .board
                .remove_piece(mov.to, captured.with_color(opp))
                .map_err(|_| InvalidMove)?;
            new_position
                .captured
                .add(captured.with_color(me))
                .map_err(|_| InvalidMove)?;
            if captured == Piece::Wazir {
                new_position.stage = Stage::End;
            }
        }
        new_position
            .board
            .place_piece(mov.to, mov.colored_piece)
            .map_err(|_| InvalidMove)?;
        new_position.to_move = opp;
        Ok(new_position)
    }
}

impl_from_str_for_parsable!(Position);

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.stage)?;
        writeln!(f, "{}", self.to_move)?;
        writeln!(f, "{}", self.captured)?;
        write!(f, "{}", self.board)?;
        Ok(())
    }
}
