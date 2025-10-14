use crate::{
    enums::{EnumMap, SimpleEnumExt},
    impl_from_str_for_parsable,
    parser::{self, ParseError, Parser, ParserExt},
    unsafe_simple_enum, Bitboard, Color, ColoredPiece, Coord, OpeningMove, Piece, PieceNonWazir,
    Square,
};
use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Stage {
    Opening,
    Regular,
    End,
}

unsafe_simple_enum!(Stage, 3);

impl Stage {
    fn parser() -> impl Parser<Output = Self> {
        parser::exact(b"opening")
            .map(|_| Stage::Opening)
            .or(parser::exact(b"regular").map(|_| Stage::Regular))
            .or(parser::exact(b"end").map(|_| Stage::End))
    }
}

impl_from_str_for_parsable!(Stage);

impl Display for Stage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Stage::Opening => write!(f, "opening"),
            Stage::Regular => write!(f, "regular"),
            Stage::End => write!(f, "end"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    stage: Stage,
    to_move: Color,
    piece_maps: EnumMap<Color, EnumMap<Piece, Bitboard>>,
    captured: EnumMap<Color, EnumMap<PieceNonWazir, u8>>,
}

impl Position {
    pub fn stage(&self) -> Stage {
        self.stage
    }

    pub fn to_move(&self) -> Color {
        self.to_move
    }

    pub fn square(&self, square: Square) -> Option<ColoredPiece> {
        for (color, piece_map) in self.piece_maps.iter() {
            for (piece, bitboard) in piece_map.iter() {
                if bitboard.contains(square) {
                    return Some(piece.with_color(color));
                }
            }
        }
        None
    }

    pub fn num_captured(&self, color: Color, piece: PieceNonWazir) -> usize {
        self.captured[color][piece].into()
    }

    pub fn parser() -> impl Parser<Output = Self> {
        Stage::parser()
            .then_ignore(parser::exact(b"\n"))
            .and(Color::parser())
            .then_ignore(parser::exact(b"\n"))
            .and(Self::captured_parser(Color::Red))
            .then_ignore(parser::exact(b"\n"))
            .and(Self::captured_parser(Color::Blue))
            .then_ignore(parser::exact(b"\n"))
            .and(Self::board_parser())
            .try_map(
                |((((stage, to_move), captured_red), captured_blue), piece_maps)| {
                    let captured = EnumMap::from_fn(|color| match color {
                        Color::Red => captured_red,
                        Color::Blue => captured_blue,
                    });
                    Self::from_parts(stage, to_move, piece_maps, captured).map_err(|_| ParseError)
                },
            )
    }

    fn captured_parser(color: Color) -> impl Parser<Output = EnumMap<PieceNonWazir, u8>> {
        ColoredPiece::parser()
            .try_map(move |cpiece| {
                if cpiece.color() == color {
                    PieceNonWazir::try_from(cpiece.piece()).map_err(|_| ParseError)
                } else {
                    Err(ParseError)
                }
            })
            .repeat(0..=Color::COUNT * OpeningMove::SIZE)
            .map(move |pieces| {
                let mut captured = EnumMap::from_fn(|_| 0);
                for piece in pieces {
                    captured[piece] += 1;
                }
                captured
            })
    }

    fn board_parser() -> impl Parser<Output = EnumMap<Color, EnumMap<Piece, Bitboard>>> {
        ColoredPiece::parser()
            .map(Some)
            .or(parser::exact(b".").map(|_| None))
            .repeat(Coord::WIDTH..=Coord::WIDTH)
            .then_ignore(parser::exact(b"\n"))
            .repeat(Coord::HEIGHT..=Coord::HEIGHT)
            .map(move |board| {
                let mut piece_maps = EnumMap::from_fn(|_| EnumMap::from_fn(|_| Bitboard::EMPTY));
                for y in 0..Coord::HEIGHT {
                    for x in 0..Coord::WIDTH {
                        let square = Coord::new(x, y).into();
                        if let Some(cpiece) = board[y][x] {
                            piece_maps[cpiece.color()][cpiece.piece()].add(square);
                        }
                    }
                }
                piece_maps
            })
    }

    fn from_parts(
        stage: Stage,
        to_move: Color,
        piece_maps: EnumMap<Color, EnumMap<Piece, Bitboard>>,
        captured: EnumMap<Color, EnumMap<PieceNonWazir, u8>>,
    ) -> Result<Position, ()> {
        match stage {
            Stage::Opening => {
                for color in Color::all() {
                    if color < to_move {
                        for (piece, &squares) in piece_maps[color].iter() {
                            if squares.count() != piece.initial_count()
                                || !squares.is_subset_of(color.initial_squares())
                            {
                                return Err(());
                            }
                        }
                    } else {
                        for (_, squares) in piece_maps[color].iter() {
                            if !squares.is_empty() {
                                return Err(());
                            }
                        }
                    }

                    for (_, &count) in captured[color].iter() {
                        if count != 0 {
                            return Err(());
                        }
                    }
                }
            }
            Stage::Regular | Stage::End => {
                for piece in PieceNonWazir::all() {
                    let mut count = 0;
                    for color in Color::all() {
                        count += piece_maps[color][piece.into()].count();
                        count += usize::from(captured[color][piece]);
                    }
                    if count != Color::COUNT * Piece::from(piece).initial_count() {
                        return Err(());
                    }
                }
                for color in Color::all() {
                    let expected_wazirs = if stage == Stage::End && color == to_move {
                        0
                    } else {
                        1
                    };
                    if piece_maps[color][Piece::Wazir].count() != expected_wazirs {
                        return Err(());
                    }
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
}

impl_from_str_for_parsable!(Position);

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.stage)?;
        writeln!(f, "{}", self.to_move)?;
        for (color, piece_counts) in self.captured.iter() {
            for (piece, &count) in piece_counts.iter() {
                let colored_piece = Piece::from(piece).with_color(color);
                for _ in 0..count {
                    write!(f, "{colored_piece}")?;
                }
            }
            writeln!(f)?;
        }
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
