use crate::{
    Bitboard, Board, Captured, Color, ColoredPiece, InvalidMove, Move, Piece, RegularMove,
    SetupMove, Square, Symmetry,
    constants::MAX_MOVES_IN_GAME,
    enums::SimpleEnumExt,
    error::Invalid,
    impl_from_str_for_parsable, movegen,
    parser::{self, ParseError, Parser, ParserExt},
    zobrist,
};
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Outcome {
    RedWin,
    Draw,
    BlueWin,
}

impl Outcome {
    pub fn parser() -> impl Parser<Output = Self> {
        parser::exact(b"red_win")
            .map(|_| Self::RedWin)
            .or(parser::exact(b"draw").map(|_| Self::Draw))
            .or(parser::exact(b"blue_win").map(|_| Self::BlueWin))
    }

    pub fn win(color: Color) -> Self {
        match color {
            Color::Red => Self::RedWin,
            Color::Blue => Self::BlueWin,
        }
    }

    pub fn red_score(self) -> i32 {
        match self {
            Self::RedWin => 1,
            Self::Draw => 0,
            Self::BlueWin => -1,
        }
    }
}

impl_from_str_for_parsable!(Outcome);

impl Display for Outcome {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Outcome::RedWin => write!(f, "red_win"),
            Outcome::Draw => write!(f, "draw"),
            Outcome::BlueWin => write!(f, "blue_win"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stage {
    Setup,
    Regular,
    End(Outcome),
}

impl Stage {
    fn parser() -> impl Parser<Output = Self> {
        parser::exact(b"setup")
            .map(|_| Stage::Setup)
            .or(parser::exact(b"regular").map(|_| Stage::Regular))
            .or(parser::exact(b"end ")
                .ignore_then(Outcome::parser())
                .map(Stage::End))
    }
}

impl_from_str_for_parsable!(Stage);

impl Display for Stage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Stage::Setup => write!(f, "setup"),
            Stage::Regular => write!(f, "regular"),
            Stage::End(outcome) => write!(f, "end {outcome}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Position {
    stage: Stage,
    move_number: u8,
    board: Board,
    captured: Captured,
}

impl Position {
    pub fn initial() -> Self {
        Self {
            stage: Stage::Setup,
            move_number: 0,
            board: Board::empty(),
            captured: Captured::new(),
        }
    }

    pub fn stage(&self) -> Stage {
        self.stage
    }

    pub fn move_number(&self) -> usize {
        self.move_number.into()
    }

    pub fn to_move(&self) -> Color {
        Color::from_index(self.move_number() % Color::COUNT)
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

    pub fn occupied_by_piece(&self, cpiece: ColoredPiece) -> Bitboard {
        self.board.occupied_by_piece(cpiece)
    }

    pub fn num_captured(&self, cpiece: ColoredPiece) -> usize {
        self.captured.get(cpiece)
    }

    pub fn hash(&self) -> u64 {
        // stage is implied by board and captured
        zobrist::TO_MOVE[self.to_move()] ^ self.board.hash() ^ self.captured.hash()
    }

    pub fn hash_ignoring_captured(&self) -> u64 {
        // There is a collision because we ignore `stage`. Setup with blue on move may look identical as a red win.
        // We ignore it, it's rare and harmless.
        zobrist::TO_MOVE[self.to_move()] ^ self.board.hash()
    }

    pub fn parser() -> impl Parser<Output = Self> {
        Stage::parser()
            .then_ignore(parser::endl())
            .and(parser::u32().try_map(|n| usize::try_from(n).map_err(|_| ParseError)))
            .then_ignore(parser::endl())
            .and(Captured::parser())
            .then_ignore(parser::endl())
            .and(Board::parser())
            .try_map(|(((stage, move_number), captured), board)| {
                Self::from_parts(stage, move_number, board, captured).map_err(|_| ParseError)
            })
    }

    fn from_parts(
        stage: Stage,
        move_number: usize,
        board: Board,
        captured: Captured,
    ) -> Result<Position, Invalid> {
        let to_move = Color::from_index(move_number % Color::COUNT);

        // Verify total piece count.
        if stage != Stage::Setup {
            for piece in Piece::all() {
                let mut count = 0;
                for color in Color::all() {
                    let cpiece = piece.with_color(color);
                    count += board.occupied_by_piece(cpiece).count();
                    count += captured.get(cpiece);
                }
                if count != piece.total_count() {
                    return Err(Invalid);
                }
            }
        }

        // Verify move number.
        match stage {
            Stage::Setup => {
                if move_number >= Color::COUNT {
                    return Err(Invalid);
                }
            }
            Stage::Regular => {
                if !(Color::COUNT..MAX_MOVES_IN_GAME).contains(&move_number) {
                    return Err(Invalid);
                }
            }
            Stage::End(Outcome::Draw) => {
                if move_number != MAX_MOVES_IN_GAME {
                    return Err(Invalid);
                }
            }
            Stage::End(outcome) => {
                if !(Color::COUNT..=MAX_MOVES_IN_GAME).contains(&move_number)
                    || outcome != Outcome::win(to_move.opposite())
                {
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
                    let squares = board.occupied_by_piece(cpiece);
                    if squares.count() != want
                        || !squares.is_subset_of(cpiece.color().initial_squares())
                        || captured.get(cpiece) != 0
                    {
                        return Err(Invalid);
                    }
                }
            }
            Stage::Regular | Stage::End(Outcome::Draw) => {
                // Verify one wazir per color on the board.
                for color in Color::all() {
                    if board
                        .occupied_by_piece(Piece::Wazir.with_color(color))
                        .count()
                        != 1
                    {
                        return Err(Invalid);
                    }
                }
            }
            Stage::End(_) => {
                // Verify opposite wazir on the board and one captured.
                let wazir_opp = Piece::Wazir.with_color(to_move.opposite());
                if board.occupied_by_piece(wazir_opp).count() != 1 || captured.get(wazir_opp) != 1 {
                    return Err(Invalid);
                }
            }
        }
        Ok(Position {
            stage,
            move_number: move_number.try_into().unwrap(),
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
        if self.stage != Stage::Setup || mov.color != me {
            return Err(InvalidMove);
        }
        mov.validate_pieces()?;
        let mut new_position = self.clone();
        let symmetry = Symmetry::pov(me).inverse();
        for (i, &piece) in mov.pieces.iter().enumerate() {
            let square = symmetry.apply(Square::from_index(i));
            new_position
                .board
                .place_piece(square, piece.with_color(me))
                .unwrap();
        }
        new_position.move_number += 1;
        if new_position.move_number() == Color::COUNT {
            new_position.stage = Stage::Regular;
        }
        Ok(new_position)
    }

    pub fn make_regular_move(&self, mov: RegularMove) -> Result<Position, InvalidMove> {
        let me = self.to_move();
        let opp = me.opposite();
        if self.stage != Stage::Regular || mov.colored_piece.color() != me {
            return Err(InvalidMove);
        }
        let mut new_position = self.clone();
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
                new_position.stage = Stage::End(Outcome::win(me));
            }
        }
        new_position
            .board
            .place_piece(mov.to, mov.colored_piece)
            .map_err(|_| InvalidMove)?;
        new_position.move_number += 1;
        if new_position.move_number() == MAX_MOVES_IN_GAME && new_position.stage == Stage::Regular {
            new_position.stage = Stage::End(Outcome::Draw);
        }
        Ok(new_position)
    }
}

impl_from_str_for_parsable!(Position);

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.stage)?;
        writeln!(f, "{}", self.move_number())?;
        writeln!(f, "{}", self.captured)?;
        write!(f, "{}", self.board)?;
        Ok(())
    }
}
