use rand::{SeedableRng, rngs::StdRng};
use wazir_drop::{Move, Player, Position, clock::Timer, movegen};

#[derive(Debug)]
pub struct RandomPlayer {
    rng: StdRng,
}

impl RandomPlayer {
    pub fn new() -> Self {
        Self {
            rng: StdRng::from_os_rng(),
        }
    }
}

impl Default for RandomPlayer {
    fn default() -> Self {
        Self::new()
    }
}

impl Player for RandomPlayer {
    fn make_move(&mut self, position: &Position, _timer: &Timer) -> Move {
        movegen::random_move(position, &mut self.rng)
    }
}
