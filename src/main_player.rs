use crate::{clock::Timer, movegen, Color, Move, Player, PlayerFactory, Position};
use std::time::Duration;

#[derive(Debug)]
pub struct MainPlayer;

impl MainPlayer {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }
}

impl Player for MainPlayer {
    fn make_move(&mut self, position: &Position, _timer: &Timer) -> Move {
        movegen::pseudomoves(position).next().expect("Stalemate")
    }
}

#[derive(Debug)]
pub struct MainPlayerFactory;

impl MainPlayerFactory {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }
}

impl PlayerFactory for MainPlayerFactory {
    fn create(
        &self,
        _game_id: &str,
        _color: Color,
        _opening: &[Move],
        _time_limit: Option<Duration>,
    ) -> Box<dyn crate::Player> {
        Box::new(MainPlayer::new())
    }
}
