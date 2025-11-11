use crate::constants::{Eval, Ply};
use std::{
    fmt::{self, Display, Formatter},
    ops::Neg,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoreExpanded {
    Win(Ply),
    Loss(Ply),
    Eval(Eval),
}

impl ScoreExpanded {
    pub fn to_relative(self, ply: Ply) -> Self {
        match self {
            Self::Win(p) => Self::Win(p.saturating_sub(ply)),
            Self::Loss(p) => Self::Loss(p.saturating_sub(ply)),
            Self::Eval(_) => self,
        }
    }

    pub fn to_absolute(self, ply: Ply) -> Self {
        match self {
            Self::Win(p) => Self::Win(p.saturating_add(ply)),
            Self::Loss(p) => Self::Loss(p.saturating_add(ply)),
            Self::Eval(_) => self,
        }
    }

    pub fn offset(self, offset: Eval) -> Self {
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
pub struct Score(Eval);

impl Score {
    pub const INFINITE: Score = Score(1000000000);
    pub const DRAW: Score = Score(0);
    const WIN_MAX_PLY: Score = Score(Self::INFINITE.0 - u8::MAX as Eval);

    pub fn next(self) -> Self {
        Self((self.0 + 1).min(Self::INFINITE.0))
    }

    pub fn prev(self) -> Self {
        Self((self.0 - 1).max(-Self::INFINITE.0))
    }

    pub fn to_relative(self, ply: Ply) -> Self {
        ScoreExpanded::from(self).to_relative(ply).into()
    }

    pub fn to_absolute(self, ply: Ply) -> Self {
        ScoreExpanded::from(self).to_absolute(ply).into()
    }

    pub fn offset(self, offset: Eval) -> Self {
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
        if score >= Score::WIN_MAX_PLY {
            Self::Win((Score::INFINITE.0 - score.0).try_into().unwrap_or(0))
        } else if score <= -Score::WIN_MAX_PLY {
            Self::Loss((Score::INFINITE.0 + score.0).try_into().unwrap_or(0))
        } else {
            Self::Eval(score.0)
        }
    }
}

impl From<ScoreExpanded> for Score {
    fn from(score: ScoreExpanded) -> Self {
        match score {
            ScoreExpanded::Win(ply) => Score(Score::INFINITE.0 - Eval::from(ply)),
            ScoreExpanded::Loss(ply) => Score(-Score::INFINITE.0 + Eval::from(ply)),
            ScoreExpanded::Eval(eval) => {
                Score(eval.clamp(-Score::WIN_MAX_PLY.0 + 1, Score::WIN_MAX_PLY.0 - 1))
            }
        }
    }
}

impl Display for Score {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", ScoreExpanded::from(*self))
    }
}
