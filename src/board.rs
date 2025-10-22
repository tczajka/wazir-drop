use crate::{
    enums::{EnumMap, SimpleEnumExt},
    error::Invalid,
    impl_from_str_for_parsable,
    parser::{self, Parser, ParserExt},
    Bitboard, Color, ColoredPiece, Coord, Square,
};
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, Copy)]
pub struct Board {
    squares: EnumMap<Square, Option<ColoredPiece>>,
    occupied_by: EnumMap<Color, Bitboard>,
    empty_squares: Bitboard,
    piece_maps: EnumMap<ColoredPiece, Bitboard>,
}

impl Board {
    pub fn empty() -> Self {
        Self {
            squares: EnumMap::from_fn(|_| None),
            occupied_by: EnumMap::from_fn(|_| Bitboard::EMPTY),
            empty_squares: !Bitboard::EMPTY,
            piece_maps: EnumMap::from_fn(|_| Bitboard::EMPTY),
        }
    }

    pub fn square(&self, square: Square) -> Option<ColoredPiece> {
        self.squares[square]
    }

    pub fn occupied_by(&self, color: Color) -> Bitboard {
        self.occupied_by[color]
    }

    pub fn empty_squares(&self) -> Bitboard {
        self.empty_squares
    }

    pub fn piece_map(&self, cpiece: ColoredPiece) -> Bitboard {
        self.piece_maps[cpiece]
    }

    pub fn place_piece(&mut self, square: Square, cpiece: ColoredPiece) -> Result<(), Invalid> {
        let s = &mut self.squares[square];
        if s.is_some() {
            return Err(Invalid);
        }
        *s = Some(cpiece);
        self.occupied_by[cpiece.color()].add(square);
        self.empty_squares.remove(square);
        self.piece_maps[cpiece].add(square);
        Ok(())
    }

    pub fn remove_piece(&mut self, square: Square, cpiece: ColoredPiece) -> Result<(), Invalid> {
        let s = &mut self.squares[square];
        if *s != Some(cpiece) {
            return Err(Invalid);
        }
        *s = None;
        self.occupied_by[cpiece.color()].remove(square);
        self.empty_squares.add(square);
        self.piece_maps[cpiece].remove(square);
        Ok(())
    }

    pub fn parser() -> impl Parser<Output = Self> {
        ColoredPiece::parser()
            .map(Some)
            .or(parser::exact(b".").map(|_| None))
            .repeat(Coord::WIDTH..=Coord::WIDTH)
            .then_ignore(parser::endl())
            .repeat(Coord::HEIGHT..=Coord::HEIGHT)
            .map(move |pieces| {
                let mut board = Board::empty();
                for square in Square::all() {
                    let coord = Coord::from(square);
                    let cpiece = pieces[coord.y()][coord.x()];
                    if let Some(cpiece) = cpiece {
                        board.place_piece(square, cpiece).unwrap();
                    }
                }
                board
            })
    }
}

impl_from_str_for_parsable!(Board);

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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
