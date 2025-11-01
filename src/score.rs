use std::{
    fmt::{self, Display, Formatter},
    ops::Neg,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoreExpanded {
    Win(usize),
    Loss(usize),
    Eval(i32),
}

impl ScoreExpanded {
    pub fn to_relative(self, move_number: usize) -> Self {
        match self {
            Self::Win(distance) => Self::Win(distance.saturating_sub(move_number)),
            Self::Loss(distance) => Self::Loss(distance.saturating_sub(move_number)),
            Self::Eval(_) => self,
        }
    }

    pub fn to_absolute(self, move_number: usize) -> Self {
        match self {
            Self::Win(distance) => Self::Win(distance.saturating_add(move_number)),
            Self::Loss(distance) => Self::Loss(distance.saturating_add(move_number)),
            Self::Eval(_) => self,
        }
    }

    pub fn offset(self, offset: i32) -> Self {
        match self {
            Self::Eval(eval) => Self::Eval(eval.saturating_add(offset)),
            _ => self,
        }
    }
}

impl Display for ScoreExpanded {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Win(distance) => write!(f, "#{}", distance),
            Self::Loss(distance) => write!(f, "-#{}", distance),
            Self::Eval(eval) => write!(f, "{}", eval),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Score(i32);

impl Score {
    pub const IMMEDIATE_WIN: Score = Score(1000000000);
    const MAX_DISTANCE: usize = 1000000;
    const WIN_MAX_DISTANCE: Score = Score(Self::IMMEDIATE_WIN.0 - Self::MAX_DISTANCE as i32);

    pub fn next(self) -> Self {
        Self((self.0 + 1).min(Self::IMMEDIATE_WIN.0))
    }

    pub fn prev(self) -> Self {
        Self((self.0 - 1).max(-Self::IMMEDIATE_WIN.0))
    }

    pub fn to_relative(self, move_number: usize) -> Self {
        ScoreExpanded::from(self).to_relative(move_number).into()
    }

    pub fn to_absolute(self, move_number: usize) -> Self {
        ScoreExpanded::from(self).to_absolute(move_number).into()
    }

    pub fn offset(self, offset: i32) -> Self {
        ScoreExpanded::from(self).offset(offset).into()
    }
}

impl Neg for Score {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl From<Score> for ScoreExpanded {
    fn from(score: Score) -> Self {
        if score >= Score::WIN_MAX_DISTANCE {
            Self::Win((Score::IMMEDIATE_WIN.0 as usize).saturating_sub(score.0 as usize))
        } else if score <= -Score::WIN_MAX_DISTANCE {
            Self::Loss((Score::IMMEDIATE_WIN.0 as usize).saturating_sub((-score.0) as usize))
        } else {
            Self::Eval(score.0)
        }
    }
}

impl From<ScoreExpanded> for Score {
    fn from(score: ScoreExpanded) -> Self {
        match score {
            ScoreExpanded::Win(distance) => {
                Score(Score::IMMEDIATE_WIN.0 - distance.min(Score::MAX_DISTANCE) as i32)
            }
            ScoreExpanded::Loss(distance) => {
                Score(-Score::IMMEDIATE_WIN.0 + distance.min(Score::MAX_DISTANCE) as i32)
            }
            ScoreExpanded::Eval(eval) => Score(eval.clamp(
                -Score::WIN_MAX_DISTANCE.0 + 1,
                Score::WIN_MAX_DISTANCE.0 - 1,
            )),
        }
    }
}

impl Display for Score {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", ScoreExpanded::from(*self))
    }
}
