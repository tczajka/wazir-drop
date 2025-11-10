use extra::moverand;
use rand::{SeedableRng, rngs::StdRng};
use std::time::Duration;
use wazir_drop::{Color, AnyMove, Player, PlayerFactory, Position, clock::Timer};

#[derive(Debug)]
pub struct RandomPlayerFactory;

impl RandomPlayerFactory {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }
}

impl PlayerFactory for RandomPlayerFactory {
    fn create(
        &self,
        _game_id: &str,
        _color: Color,
        _opening: &[AnyMove],
        _time_limit: Option<Duration>,
    ) -> Box<dyn Player> {
        Box::new(RandomPlayer::new())
    }
}

#[derive(Debug)]
pub struct RandomPlayer {
    rng: StdRng,
}

impl RandomPlayer {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            rng: StdRng::from_os_rng(),
        }
    }
}

impl Player for RandomPlayer {
    fn make_move(&mut self, position: &Position, _timer: &Timer) -> AnyMove {
        moverand::random_move(position, &mut self.rng)
    }
}
