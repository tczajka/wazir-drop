use rand::{SeedableRng, rngs::StdRng, seq::IteratorRandom};
use wazir_drop::{Color, Move, Player, Position, movegen};

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

    fn make_move(&mut self, position: &Position) -> Move {
        movegen::pseudomoves(position)
            .choose(&mut self.rng)
            .expect("Stalemate")
    }
}
