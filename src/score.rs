use std::{
    fmt::{self, Display, Formatter},
    ops::Neg,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Score(i32);

impl Score {
    pub const IMMEDIATE_WIN: Score = Score(1000000000);
    const MAX_PLY: usize = 1000000;
    pub const MIN_WIN: Score = Self::win_in(Self::MAX_PLY);
    const MAX_EVAL: i32 = Self::MIN_WIN.prev().0;

    pub const fn win_in(ply: usize) -> Self {
        let ply = if ply > Self::MAX_PLY {
            Self::MAX_PLY
        } else {
            ply
        };
        Self(Self::IMMEDIATE_WIN.0 - ply as i32)
    }

    pub fn lose_in(ply: usize) -> Self {
        -Self::win_in(ply)
    }

    pub const fn next(self) -> Self {
        Self(self.0 + 1)
    }

    pub const fn prev(self) -> Self {
        Self(self.0 - 1)
    }

    pub fn from_eval(value: i32) -> Self {
        Self(value.clamp(-Self::MAX_EVAL, Self::MAX_EVAL))
    }

    // Back by one ply, from the other side's perspective.
    pub fn back(self) -> Self {
        if self > Self::MIN_WIN {
            -self.prev()
        } else if self < -Self::MIN_WIN {
            -self.next()
        } else {
            -self
        }
    }

    // Forward by one ply.
    pub fn forward(self) -> Self {
        if self >= Self::MIN_WIN {
            -self.next()
        } else if self <= -Self::MIN_WIN {
            -self.prev()
        } else {
            -self
        }
    }
}

impl Neg for Score {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl Display for Score {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if *self >= Self::MIN_WIN {
            write!(f, "#{}", Self::IMMEDIATE_WIN.0 - self.0)
        } else if *self <= -Self::MIN_WIN {
            write!(f, "-#{}", Self::IMMEDIATE_WIN.0 + self.0)
        } else {
            write!(f, "{}", self.0)
        }
    }
}
