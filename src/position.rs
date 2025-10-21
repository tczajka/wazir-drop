use crate::{
    enums::{EnumMap, SimpleEnumExt},
    impl_from_str_for_parsable, movegen,
    parser::{self, ParseError, Parser, ParserExt},
    unsafe_simple_enum, Bitboard, Color, ColoredPiece, Coord, InvalidMove, Move, Piece,
    RegularMove, SetupMove, ShortMove, ShortMoveFrom, Square,
};
use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

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
pub struct InvalidPosition;

#[derive(Debug, Clone, Copy)]
pub struct Position {
    stage: Stage,
    to_move: Color,
    piece_maps: EnumMap<ColoredPiece, Bitboard>,
    captured: EnumMap<ColoredPiece, u8>,
}

impl Position {
    pub fn initial() -> Self {
        Self::from_parts(
            Stage::Setup,
            Color::Red,
            EnumMap::from_fn(|_| Bitboard::EMPTY),
            EnumMap::from_fn(|_| 0),
        )
        .unwrap()
    }

    pub fn stage(&self) -> Stage {
        self.stage
    }

    pub fn to_move(&self) -> Color {
        self.to_move
    }

    pub fn square(&self, square: Square) -> Option<ColoredPiece> {
        for (cpiece, &bitboard) in self.piece_maps.iter() {
            if bitboard.contains(square) {
                return Some(cpiece);
            }
        }
        None
    }

    pub fn piece_map(&self, cpiece: ColoredPiece) -> Bitboard {
        self.piece_maps[cpiece]
    }

    pub fn occupied_by(&self, color: Color) -> Bitboard {
        let mut bitboard = Bitboard::EMPTY;
        for piece in Piece::all() {
            bitboard |= self.piece_map(piece.with_color(color));
        }
        bitboard
    }

    pub fn empty_squares(&self) -> Bitboard {
        let mut occupied = Bitboard::EMPTY;
        for color in Color::all() {
            occupied |= self.occupied_by(color);
        }
        !occupied
    }

    pub fn num_captured(&self, cpiece: ColoredPiece) -> usize {
        self.captured[cpiece].into()
    }

    pub fn parser() -> impl Parser<Output = Self> {
        Stage::parser()
            .then_ignore(parser::exact(b"\n"))
            .and(Color::parser())
            .then_ignore(parser::exact(b"\n"))
            .and(Self::captured_parser())
            .then_ignore(parser::exact(b"\n"))
            .and(Self::board_parser())
            .try_map(|(((stage, to_move), captured), piece_maps)| {
                Self::from_parts(stage, to_move, piece_maps, captured).map_err(|_| ParseError)
            })
    }

    fn captured_parser() -> impl Parser<Output = EnumMap<ColoredPiece, u8>> {
        ColoredPiece::parser()
            .repeat(0..=Color::COUNT * SetupMove::SIZE)
            .map(move |pieces| {
                let mut captured = EnumMap::from_fn(|_| 0);
                for piece in pieces {
                    captured[piece] += 1;
                }
                captured
            })
    }

    fn board_parser() -> impl Parser<Output = EnumMap<ColoredPiece, Bitboard>> {
        ColoredPiece::parser()
            .map(Some)
            .or(parser::exact(b".").map(|_| None))
            .repeat(Coord::WIDTH..=Coord::WIDTH)
            .then_ignore(parser::exact(b"\n"))
            .repeat(Coord::HEIGHT..=Coord::HEIGHT)
            .map(move |board| {
                let mut piece_maps = EnumMap::from_fn(|_| Bitboard::EMPTY);
                for (y, line) in board.iter().enumerate() {
                    for (x, &optional_cpiece) in line.iter().enumerate() {
                        let square = Coord::new(x, y).into();
                        if let Some(cpiece) = optional_cpiece {
                            piece_maps[cpiece].add(square);
                        }
                    }
                }
                piece_maps
            })
    }

    fn from_parts(
        stage: Stage,
        to_move: Color,
        piece_maps: EnumMap<ColoredPiece, Bitboard>,
        captured: EnumMap<ColoredPiece, u8>,
    ) -> Result<Position, InvalidPosition> {
        // Verify total piece count.
        if stage != Stage::Setup {
            for piece in Piece::all() {
                let mut count = 0;
                for color in Color::all() {
                    let cpiece = piece.with_color(color);
                    count += piece_maps[cpiece].count();
                    count += usize::from(captured[cpiece]);
                }
                if count != Color::COUNT * piece.initial_count() {
                    return Err(InvalidPosition);
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
                    let squares = piece_maps[cpiece];
                    if squares.count() != want
                        || !squares.is_subset_of(cpiece.color().initial_squares())
                        || captured[cpiece] != 0
                    {
                        return Err(InvalidPosition);
                    }
                }
            }
            Stage::Regular => {
                // Verify one wazir per color on the board.
                for color in Color::all() {
                    if piece_maps[Piece::Wazir.with_color(color)].count() != 1 {
                        return Err(InvalidPosition);
                    }
                }
            }
            Stage::End => {
                // Verify opposite wazir on the board and one captured.
                let wazir_opp = Piece::Wazir.with_color(to_move.opposite());
                if piece_maps[wazir_opp].count() != 1 || captured[wazir_opp] != 1 {
                    return Err(InvalidPosition);
                }
            }
        }
        Ok(Position {
            stage,
            to_move,
            piece_maps,
            captured,
        })
    }

    pub fn move_from_short_move(&self, short_move: ShortMove) -> Result<Move, InvalidMove> {
        match short_move {
            ShortMove::Setup(mov) => {
                if self.stage != Stage::Setup || mov.color != self.to_move {
                    return Err(InvalidMove);
                }
                mov.validate_pieces()?;
                Ok(Move::Setup(mov))
            }
            ShortMove::Regular { from, to } => {
                if self.stage != Stage::Regular {
                    return Err(InvalidMove);
                }

                let captured = match self.square(to) {
                    None => None,
                    Some(captured) => {
                        if captured.color() != self.to_move.opposite() {
                            return Err(InvalidMove);
                        }
                        Some(captured.piece())
                    }
                };

                let (colored_piece, from) = match from {
                    ShortMoveFrom::Piece(cpiece) => {
                        if captured.is_some() || self.num_captured(cpiece) == 0 {
                            return Err(InvalidMove);
                        }
                        (cpiece, None)
                    }
                    ShortMoveFrom::Square(square) => {
                        let piece = self.square(square).ok_or(InvalidMove)?;
                        movegen::validate_from_to(piece.piece(), square, to)?;
                        (piece, Some(square))
                    }
                };

                if colored_piece.color() != self.to_move {
                    return Err(InvalidMove);
                }
                Ok(Move::Regular(RegularMove {
                    colored_piece,
                    from,
                    captured,
                    to,
                }))
            }
        }
    }

    pub fn make_move(&self, mov: Move) -> Result<Position, InvalidMove> {
        match mov {
            Move::Setup(mov) => self.make_setup_move(mov),
            Move::Regular(mov) => self.make_regular_move(mov),
        }
    }

    pub fn make_setup_move(&self, mov: SetupMove) -> Result<Position, InvalidMove> {
        if self.stage != Stage::Setup || mov.color != self.to_move {
            return Err(InvalidMove);
        }
        mov.validate_pieces()?;
        let mut new_position = *self;
        for (i, &piece) in mov.pieces.iter().enumerate() {
            let square = Square::from_index(i).pov(mov.color);
            new_position.piece_maps[piece.with_color(mov.color)].add(square);
        }
        new_position.to_move = new_position.to_move.opposite();
        if new_position.to_move == Color::Red {
            new_position.stage = Stage::Regular;
        }
        Ok(new_position)
    }

    pub fn make_regular_move(&self, mov: RegularMove) -> Result<Position, InvalidMove> {
        let color = self.to_move;
        let piece = mov.colored_piece.piece();
        if self.stage != Stage::Regular || mov.colored_piece.color() != color {
            return Err(InvalidMove);
        }
        let mut new_position = *self;
        match mov.from {
            None => {
                let count = &mut new_position.captured[mov.colored_piece];
                *count = count.checked_sub(1).ok_or(InvalidMove)?;
            }
            Some(from) => {
                movegen::validate_from_to(piece, from, mov.to)?;
                let map = &mut new_position.piece_maps[mov.colored_piece];
                if !map.contains(from) {
                    return Err(InvalidMove);
                }
                map.remove(from);
            }
        }
        match mov.captured {
            None => {
                if self.square(mov.to).is_some() {
                    return Err(InvalidMove);
                }
            }
            Some(captured) => {
                let map = &mut new_position.piece_maps[captured.with_color(color.opposite())];
                if !map.contains(mov.to) {
                    return Err(InvalidMove);
                }
                map.remove(mov.to);
                new_position.captured[captured.with_color(color)] += 1;
                if captured == Piece::Wazir {
                    new_position.stage = Stage::End;
                }
            }
        }
        new_position.piece_maps[mov.colored_piece].add(mov.to);
        new_position.to_move = color.opposite();
        Ok(new_position)
    }
}

impl_from_str_for_parsable!(Position);

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.stage)?;
        writeln!(f, "{}", self.to_move)?;
        for (cpiece, &count) in self.captured.iter() {
            for _ in 0..count {
                write!(f, "{cpiece}")?;
            }
        }
        writeln!(f)?;
        for y in 0..Coord::HEIGHT {
            for x in 0..Coord::WIDTH {
                let square = Coord::new(x, y).into();
                match self.square(square) {
                    None => write!(f, ".")?,
                    Some(cpiece) => write!(f, "{cpiece}")?,
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
