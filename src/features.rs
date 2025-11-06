use crate::{either::Either, Color, Move, Position, RegularMove, SetupMove};
use std::fmt::Debug;

pub trait Features: Debug + Copy + Send + 'static {
    fn count(self) -> usize;

    fn all(self, position: &Position, color: Color) -> impl Iterator<Item = usize>;

    /// Returns (added features, removed features).
    ///
    /// If it's too complicated, returns `None`. Caller should fall back to `all_features`.
    fn diff(
        self,
        mov: Move,
        new_position: &Position,
        color: Color,
    ) -> Option<(impl Iterator<Item = usize>, impl Iterator<Item = usize>)> {
        match mov {
            Move::Setup(mov) => self
                .diff_setup(mov, new_position, color)
                .map(|(added, removed)| (Either::Left(added), Either::Left(removed))),
            Move::Regular(mov) => self
                .diff_regular(mov, new_position, color)
                .map(|(added, removed)| (Either::Right(added), Either::Right(removed))),
        }
    }

    fn diff_setup(
        self,
        mov: SetupMove,
        new_position: &Position,
        color: Color,
    ) -> Option<(impl Iterator<Item = usize>, impl Iterator<Item = usize>)>;

    fn diff_regular(
        self,
        mov: RegularMove,
        new_position: &Position,
        color: Color,
    ) -> Option<(impl Iterator<Item = usize>, impl Iterator<Item = usize>)>;

    /// Redundant feature sets that always sum to a constant.
    fn redundant(self) -> impl Iterator<Item = impl Iterator<Item = (usize, i32)>>;
}
