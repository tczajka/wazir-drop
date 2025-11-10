use crate::{clock::Timer, AnyMove, Color, Position};
use std::time::Duration;

/// It can play a single game.
pub trait Player {
    fn opponent_move(&mut self, _position: &Position, _mov: AnyMove, _timer: &Timer) {}
    fn make_move(&mut self, position: &Position, timer: &Timer) -> AnyMove;
}

/// It can create players.
pub trait PlayerFactory: Send + Sync {
    fn create(
        &self,
        game_id: &str,
        color: Color,
        opening: &[AnyMove],
        time_limit: Option<Duration>,
    ) -> Box<dyn Player>;
}
