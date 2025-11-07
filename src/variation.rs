use crate::{smallvec::SmallVec, RegularMove};
use std::{
    fmt::{self, Display, Formatter},
    ops::Deref,
};

pub trait Variation {
    fn empty() -> Self;
    fn empty_truncated() -> Self;
}

pub trait ExtendableVariation: Variation {
    type Extended: NonEmptyVariation<Truncated = Self>;
    fn add_front(self, mov: RegularMove) -> Self::Extended;
}

pub trait NonEmptyVariation: Variation {
    type Truncated: ExtendableVariation<Extended = Self>;
    fn first(&self) -> Option<RegularMove>;
    fn truncate(self) -> Self::Truncated;
}

pub struct LongVariation {
    pub moves: SmallVec<RegularMove, { Self::MAX_LENGTH }>,
    pub truncated: bool,
}

impl LongVariation {
    pub const MAX_LENGTH: usize = 100;
}

impl Deref for LongVariation {
    type Target = [RegularMove];

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

    fn add_front(self, mov: RegularMove) -> Self {
        let mut res = Self::empty();
        res.moves.push(mov);
        for &mov in self.moves.iter() {
            if res.moves.len() >= Self::MAX_LENGTH {
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
}

impl NonEmptyVariation for LongVariation {
    type Truncated = Self;

    fn truncate(self) -> Self::Truncated {
        self
    }

    fn first(&self) -> Option<RegularMove> {
        self.moves.first().copied()
    }
}

#[derive(Debug)]
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

    fn add_front(self, mov: RegularMove) -> Self::Extended {
        OneMoveVariation { mov: Some(mov) }
    }
}

#[derive(Debug)]
pub struct OneMoveVariation {
    mov: Option<RegularMove>,
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

    fn first(&self) -> Option<RegularMove> {
        self.mov
    }
}
