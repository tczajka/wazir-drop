use crate::{Color, Move, Position};

pub trait Player {
    fn new(color: Color, opening: &[Move]) -> Self;
    fn opponent_move(&mut self, _position: &Position, _mov: Move) {}
    fn make_move(&mut self, position: &Position) -> Move;
}
