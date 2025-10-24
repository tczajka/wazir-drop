use crate::{clock::Timer, Color, Move, Position};
use std::time::Duration;

/// It can play a single game.
pub trait Player {
    fn opponent_move(&mut self, _position: &Position, _mov: Move, _timer: &Timer) {}
    fn make_move(&mut self, position: &Position, timer: &Timer) -> Move;
}

/// It can create players.
pub trait PlayerFactory {
    fn create(
        &self,
        game_id: &str,
        color: Color,
        opening: &[Move],
        time_limit: Option<Duration>,
    ) -> Box<dyn Player>;
}
