#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Score(i32);

impl Score {
    const WIN: i32 = 1000000000;
    const MAX_DEPTH: i32 = 1000000;
    const MAX_EVAL: i32 = Self::WIN - Self::MAX_DEPTH - 1;

    pub fn from_eval(value: i32) -> Self {
        Self(value.clamp(-Self::MAX_EVAL, Self::MAX_EVAL))
    }
}
