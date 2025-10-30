use crate::{
    Color, ColoredPiece, Piece, SetupMove,
    enums::{EnumMap, SimpleEnumExt},
    error::Invalid,
    impl_from_str_for_parsable,
    parser::{ParseError, Parser, ParserExt},
    zobrist,
};
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, Copy)]
pub struct CapturedOneSide {
    counts: EnumMap<Piece, u8>,
}

impl CapturedOneSide {
    pub fn new() -> Self {
        Self {
            counts: EnumMap::from_fn(|_| 0),
        }
    }

    pub fn get(&self, piece: Piece) -> usize {
        self.counts[piece].into()
    }

    pub fn add(&mut self, piece: Piece) -> Result<(), Invalid> {
        let c = &mut self.counts[piece];
        let count = usize::from(*c);
        if count >= piece.total_count() {
            return Err(Invalid);
        }
        *c += 1;
        Ok(())
    }

    pub fn remove(&mut self, piece: Piece) -> Result<(), Invalid> {
        let c = &mut self.counts[piece];
        if *c == 0 {
            return Err(Invalid);
        }
        *c -= 1;
        Ok(())
    }
}

impl Default for CapturedOneSide {
    fn default() -> Self {
        Self::new()
    }
}

/// Allows capturing up to `piece::total_count()` of each ColoredPiece.
#[derive(Debug, Clone, Copy)]
pub struct Captured {
    sides: EnumMap<Color, CapturedOneSide>,
    hash: u64,
}

impl Captured {
    pub fn new() -> Self {
        Self {
            sides: EnumMap::from_fn(|_| CapturedOneSide::new()),
            hash: 0,
        }
    }

    pub fn get(&self, cpiece: ColoredPiece) -> usize {
        self.sides[cpiece.color()].get(cpiece.piece())
    }

    pub fn hash(&self) -> u64 {
        self.hash
    }

    pub fn add(&mut self, cpiece: ColoredPiece) -> Result<(), Invalid> {
        let color = cpiece.color();
        let piece = cpiece.piece();
        self.sides[color].add(piece)?;
        self.hash ^= zobrist::captured(cpiece, self.sides[color].get(piece) - 1);
        Ok(())
    }

    pub fn remove(&mut self, cpiece: ColoredPiece) -> Result<(), Invalid> {
        let color = cpiece.color();
        let piece = cpiece.piece();
        self.sides[color].remove(piece)?;
        self.hash ^= zobrist::captured(cpiece, self.sides[color].get(piece));
        Ok(())
    }

    pub fn parser() -> impl Parser<Output = Self> {
        ColoredPiece::parser()
            .repeat(0..=Color::COUNT * SetupMove::SIZE)
            .try_map(move |pieces| {
                let mut captured = Self::new();
                for piece in pieces {
                    captured.add(piece).map_err(|_| ParseError)?;
                }
                Ok(captured)
            })
    }
}

impl_from_str_for_parsable!(Captured);

impl Default for Captured {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for Captured {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for (color, captured_by_color) in self.sides.iter() {
            for piece in Piece::all() {
                let count = captured_by_color.get(piece);
                for _ in 0..count {
                    write!(f, "{}", piece.with_color(color))?;
                }
            }
        }
        Ok(())
    }
}

/// Not counting Wazirs.
pub const NUM_CAPTURED_INDEXES: usize = Color::COUNT * SetupMove::SIZE;

pub fn captured_index(piece: Piece, index: usize) -> usize {
    CAPTURED_OFFSET_TABLE[piece] + index
}

static CAPTURED_OFFSET_TABLE: EnumMap<Piece, usize> = {
    let mut table = [0; Piece::COUNT];
    let mut sum = 0;
    let mut piece_idx = 0;
    while piece_idx != Piece::COUNT {
        table[piece_idx] = sum;
        sum += Piece::from_index(piece_idx).total_count();
        piece_idx += 1;
    }
    assert!(sum == NUM_CAPTURED_INDEXES);
    EnumMap::from_array(table)
};
