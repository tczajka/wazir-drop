use crate::{either::Either, AnyMove, Color, Move, Position, SetupMove};
use std::fmt::Debug;

pub trait Features: Debug + Copy + Send + Sync + 'static {
    fn count(self) -> usize;
    fn approximate_avg_set(self) -> f64;

    fn all(self, position: &Position, color: Color) -> impl Iterator<Item = usize>;

    /// Returns (added features, removed features).
    ///
    /// If it's too complicated, returns `None`. Caller should fall back to `all_features`.
    fn diff_any(
        self,
        mov: AnyMove,
        new_position: &Position,
        color: Color,
    ) -> Option<(impl Iterator<Item = usize>, impl Iterator<Item = usize>)> {
        match mov {
            AnyMove::Setup(mov) => self
                .diff_setup(mov, new_position, color)
                .map(|(added, removed)| (Either::Left(added), Either::Left(removed))),
            AnyMove::Regular(mov) => self
                .diff(mov, new_position, color)
                .map(|(added, removed)| (Either::Right(added), Either::Right(removed))),
        }
    }

    fn diff_setup(
        self,
        mov: SetupMove,
        new_position: &Position,
        color: Color,
    ) -> Option<(impl Iterator<Item = usize>, impl Iterator<Item = usize>)>;

    fn diff(
        self,
        mov: Move,
        new_position: &Position,
        color: Color,
    ) -> Option<(impl Iterator<Item = usize>, impl Iterator<Item = usize>)>;
}
