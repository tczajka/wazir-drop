use rand::{SeedableRng, rngs::StdRng};
use wazir_drop::{Color, Move, Player, Position, clock::Timer, movegen};

#[derive(Debug)]
pub struct RandomPlayer {
    rng: StdRng,
}

impl Player for RandomPlayer {
    fn new(_color: Color, _opening: &[Move]) -> Self {
        Self {
            rng: StdRng::from_os_rng(),
        }
    }

    fn make_move(&mut self, position: &Position, _timer: &Timer) -> Move {
        movegen::random_move(position, &mut self.rng)
    }
}
