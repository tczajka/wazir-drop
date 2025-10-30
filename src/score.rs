use std::{
    fmt::{self, Display, Formatter},
    ops::Neg,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Score(i32);

impl Score {
    pub const IMMEDIATE_WIN: Score = Score(1000000000);
    pub const TOO_LONG: usize = 1000000;
    pub const WIN_TOO_LONG: Score = Score(Self::IMMEDIATE_WIN.0 - Self::TOO_LONG as i32);

    pub fn win(move_number: usize) -> Self {
        if move_number > Self::TOO_LONG {
            Self::WIN_TOO_LONG
        } else {
            Self(Self::IMMEDIATE_WIN.0 - move_number as i32)
        }
    }

    pub fn loss(move_number: usize) -> Self {
        -Self::win(move_number)
    }

    pub fn next(self) -> Self {
        Self(self.0 + 1)
    }

    pub fn prev(self) -> Self {
        Self(self.0 - 1)
    }

    pub fn from_eval(value: i32) -> Self {
        assert!(value < Self::WIN_TOO_LONG.0 && value > -Self::WIN_TOO_LONG.0);
        Self(value)
    }

    pub fn to_relative(self, move_number: usize) -> Self {
        if self > Self::WIN_TOO_LONG {
            Self(self.0 + move_number as i32).min(Self::IMMEDIATE_WIN)
        } else if self < -Self::WIN_TOO_LONG {
            Self(self.0 - move_number as i32).max(-Self::IMMEDIATE_WIN)
        } else {
            self
        }
    }

    pub fn to_absolute(self, move_number: usize) -> Self {
        if self > Self::WIN_TOO_LONG {
            Self(self.0 - move_number as i32).max(Self::WIN_TOO_LONG)
        } else if self < -Self::WIN_TOO_LONG {
            Self(self.0 + move_number as i32).min(-Self::WIN_TOO_LONG)
        } else {
            self
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
        if *self >= Self::WIN_TOO_LONG {
            write!(f, "#{}", Self::IMMEDIATE_WIN.0 - self.0)
        } else if *self <= -Self::WIN_TOO_LONG {
            write!(f, "-#{}", Self::IMMEDIATE_WIN.0 + self.0)
        } else {
            write!(f, "{}", self.0)
        }
    }
}
