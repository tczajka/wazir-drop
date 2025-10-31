use crate::{
    enums::EnumMap,
    impl_from_str_for_parsable,
    parser::{self, ParseError, Parser, ParserExt},
    Color, ColoredPiece, Piece, Square,
};
use std::{
    array,
    fmt::{self, Display, Formatter},
    mem,
};

#[derive(Debug, Clone, Copy)]
pub struct InvalidMove;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SetupMove {
    pub color: Color,
    // From square 0 or square 63.
    pub pieces: [Piece; Self::SIZE],
}

impl SetupMove {
    pub const SIZE: usize = 16;

    fn parser() -> impl Parser<Output = Self> {
        ColoredPiece::parser()
            .repeat(SetupMove::SIZE..=SetupMove::SIZE)
            .try_map(|colored_pieces| {
                let color = colored_pieces[0].color();
                if colored_pieces.iter().any(|p| p.color() != color) {
                    return Err(ParseError);
                }
                let mut pieces = array::from_fn(|i| colored_pieces[i].piece());
                match color {
                    Color::Red => {}
                    Color::Blue => {
                        pieces.reverse();
                    }
                }
                Ok(SetupMove { color, pieces })
            })
    }

    pub fn validate_pieces(&self) -> Result<(), InvalidMove> {
        let mut counts = EnumMap::from_fn(|_| 0);
        for piece in self.pieces {
            counts[piece] += 1;
        }
        if counts
            .iter()
            .any(|(piece, &count)| count != piece.initial_count())
        {
            return Err(InvalidMove);
        }
        Ok(())
    }
}

impl_from_str_for_parsable!(SetupMove);

impl Display for SetupMove {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let color = self.color;
        let mut pieces = self.pieces;
        match color {
            Color::Red => {}
            Color::Blue => {
                pieces.reverse();
            }
        }
        for piece in pieces {
            write!(f, "{}", piece.with_color(color))?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(align(4))]
pub struct RegularMove {
    pub colored_piece: ColoredPiece,
    pub from: Option<Square>,
    pub captured: Option<Piece>,
    pub to: Square,
}

const _: () = assert!(mem::size_of::<RegularMove>() == 4);

impl RegularMove {
    pub fn parser() -> impl Parser<Output = Self> {
        ColoredPiece::parser()
            .and_then(move |cpiece| {
                parser::exact(b"@")
                    .map(|_| (None, None)) // (from, captured)
                    .or(Square::parser().map(Some).and(
                        parser::exact(b"-").map(|_| None).or(parser::exact(b"x")
                            .ignore_then(ColoredPiece::parser())
                            .try_map(move |cpiece2| {
                                if cpiece2.color() != cpiece.color().opposite() {
                                    return Err(ParseError);
                                }
                                Ok(Some(cpiece2.piece()))
                            })),
                    ))
                    .map(move |(from, captured)| (cpiece, from, captured))
            })
            .and(Square::parser())
            .map(|((colored_piece, from, captured), to)| RegularMove {
                colored_piece,
                from,
                captured,
                to,
            })
    }
}

impl_from_str_for_parsable!(RegularMove);

impl Display for RegularMove {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.colored_piece)?;
        match (self.from, self.captured) {
            (None, None) => write!(f, "@")?,
            (None, Some(_)) => panic!("Drop capture"),
            (Some(from), None) => write!(f, "{from}-")?,
            (Some(from), Some(captured)) => {
                let captured_piece = captured.with_color(self.colored_piece.color().opposite());
                write!(f, "{from}x{captured_piece}")?;
            }
        }
        write!(f, "{}", self.to)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Move {
    Setup(SetupMove),
    Regular(RegularMove),
}

impl Move {
    pub fn parser() -> impl Parser<Output = Self> {
        SetupMove::parser()
            .map(Move::from)
            .or(RegularMove::parser().map(Move::from))
    }
}

impl_from_str_for_parsable!(Move);

impl From<SetupMove> for Move {
    fn from(mov: SetupMove) -> Self {
        Move::Setup(mov)
    }
}

impl From<RegularMove> for Move {
    fn from(mov: RegularMove) -> Self {
        Move::Regular(mov)
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Move::Setup(mov) => write!(f, "{mov}"),
            Move::Regular(mov) => write!(f, "{mov}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShortMoveFrom {
    Piece(ColoredPiece),
    Square(Square),
}

impl ShortMoveFrom {
    pub fn parser() -> impl Parser<Output = Self> {
        Square::parser()
            .map(ShortMoveFrom::Square)
            .or(ColoredPiece::parser().map(ShortMoveFrom::Piece))
    }
}

impl Display for ShortMoveFrom {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            ShortMoveFrom::Piece(cpiece) => write!(f, "{cpiece}"),
            ShortMoveFrom::Square(square) => write!(f, "{square}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShortMove {
    Setup(SetupMove),
    Regular { from: ShortMoveFrom, to: Square },
}

impl ShortMove {
    pub fn parser() -> impl Parser<Output = Self> {
        SetupMove::parser()
            .map(ShortMove::Setup)
            .or(ShortMoveFrom::parser()
                .and(Square::parser())
                .map(|(from, to)| ShortMove::Regular { from, to }))
    }
}

impl_from_str_for_parsable!(ShortMove);

impl Display for ShortMove {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ShortMove::Setup(mov) => write!(f, "{mov}"),
            ShortMove::Regular { from, to } => write!(f, "{from}{to}"),
        }
    }
}

impl From<Move> for ShortMove {
    fn from(mov: Move) -> Self {
        match mov {
            Move::Setup(mov) => ShortMove::Setup(mov),
            Move::Regular(mov) => ShortMove::Regular {
                from: match mov.from {
                    None => ShortMoveFrom::Piece(mov.colored_piece),
                    Some(from) => ShortMoveFrom::Square(from),
                },
                to: mov.to,
            },
        }
    }
}
