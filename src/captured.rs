use crate::{
    enums::EnumMap,
    error::Invalid,
    impl_from_str_for_parsable,
    parser::{ParseError, Parser, ParserExt},
    Color, ColoredPiece, SetupMove,
};
use std::fmt::{self, Display, Formatter};

/// Allows capturing up to `Color::COUNT * piece::initial_count()` of each ColoredPiece.
#[derive(Debug, Clone, Copy)]
pub struct Captured {
    counts: EnumMap<ColoredPiece, u8>,
}

impl Captured {
    pub fn new() -> Self {
        Self {
            counts: EnumMap::from_fn(|_| 0),
        }
    }

    pub fn get(&self, cpiece: ColoredPiece) -> usize {
        self.counts[cpiece].into()
    }

    pub fn add(&mut self, cpiece: ColoredPiece) -> Result<(), Invalid> {
        let c = &mut self.counts[cpiece];
        if usize::from(*c) >= Color::COUNT * cpiece.piece().initial_count() {
            return Err(Invalid);
        }
        *c += 1;
        Ok(())
    }

    pub fn remove(&mut self, cpiece: ColoredPiece) -> Result<(), Invalid> {
        let c = &mut self.counts[cpiece];
        if *c == 0 {
            return Err(Invalid);
        }
        *c -= 1;
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
        for (cpiece, &count) in self.counts.iter() {
            for _ in 0..count {
                write!(f, "{cpiece}")?;
            }
        }
        Ok(())
    }
}
