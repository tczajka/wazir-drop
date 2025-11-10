use crate::{constants::MAX_VARIATION_LENGTH, smallvec::SmallVec, PVTable, Move};
use std::{
    fmt::{self, Display, Formatter},
    ops::Deref,
};

pub trait Variation: Clone {
    fn empty() -> Self;
    fn empty_truncated() -> Self;
}

pub trait ExtendableVariation: Variation {
    type Extended: NonEmptyVariation<Truncated = Self>;
    fn add_front(self, mov: Move) -> Self::Extended;
    fn pvtable_get(pvtable: &mut PVTable, hash: u64) -> Option<Self>;
    fn pvtable_set(pvtable: &mut PVTable, hash: u64, variation: Self);
}

pub trait NonEmptyVariation: Variation {
    type Truncated: ExtendableVariation<Extended = Self>;
    fn first(&self) -> Option<Move>;
    fn truncate(self) -> Self::Truncated;
}

#[derive(Clone)]
pub struct LongVariation {
    pub moves: SmallVec<Move, MAX_VARIATION_LENGTH>,
    pub truncated: bool,
}

impl Default for LongVariation {
    fn default() -> Self {
        Self::empty()
    }
}

impl Deref for LongVariation {
    type Target = [Move];

    fn deref(&self) -> &Self::Target {
        &self.moves
    }
}

impl Display for LongVariation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for (index, &mov) in self.moves.iter().enumerate() {
            if index != 0 {
                write!(f, " ")?;
            }
            write!(f, "{mov}")?;
        }
        if self.truncated {
            write!(f, " (trunc)")?;
        }
        Ok(())
    }
}

impl Variation for LongVariation {
    fn empty() -> Self {
        Self {
            moves: SmallVec::new(),
            truncated: false,
        }
    }

    fn empty_truncated() -> Self {
        Self {
            moves: SmallVec::new(),
            truncated: true,
        }
    }
}

impl ExtendableVariation for LongVariation {
    type Extended = Self;

    fn add_front(self, mov: Move) -> Self {
        let mut res = Self::empty();
        res.moves.push(mov);
        for &mov in self.moves.iter() {
            if res.moves.len() >= MAX_VARIATION_LENGTH {
                res.truncated = true;
                break;
            }
            res.moves.push(mov);
        }

        if self.truncated {
            res.truncated = true;
        }

        res
    }

    fn pvtable_get(pvtable: &mut PVTable, hash: u64) -> Option<Self> {
        pvtable.get(hash)
    }

    fn pvtable_set(pvtable: &mut PVTable, hash: u64, variation: Self) {
        pvtable.set(hash, variation);
    }
}

impl NonEmptyVariation for LongVariation {
    type Truncated = Self;

    fn truncate(self) -> Self::Truncated {
        self
    }

    fn first(&self) -> Option<Move> {
        self.moves.first().copied()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct EmptyVariation;

impl Variation for EmptyVariation {
    fn empty() -> Self {
        Self
    }
    fn empty_truncated() -> Self {
        Self
    }
}

impl ExtendableVariation for EmptyVariation {
    type Extended = OneMoveVariation;

    fn add_front(self, mov: Move) -> Self::Extended {
        OneMoveVariation { mov: Some(mov) }
    }

    fn pvtable_get(_pvtable: &mut PVTable, _hash: u64) -> Option<Self> {
        None
    }

    fn pvtable_set(_pvtable: &mut PVTable, _hash: u64, _variation: Self) {}
}

#[derive(Debug, Copy, Clone)]
pub struct OneMoveVariation {
    mov: Option<Move>,
}

impl Variation for OneMoveVariation {
    fn empty() -> Self {
        Self { mov: None }
    }

    fn empty_truncated() -> Self {
        Self { mov: None }
    }
}

impl NonEmptyVariation for OneMoveVariation {
    type Truncated = EmptyVariation;

    fn truncate(self) -> Self::Truncated {
        EmptyVariation
    }

    fn first(&self) -> Option<Move> {
        self.mov
    }
}
