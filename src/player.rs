use crate::{clock::Timer, Color, Move, Position};

pub trait Player {
    fn init(&mut self, _color: Color, _opening: &[Move], _timer: &Timer) {}
    fn opponent_move(&mut self, _position: &Position, _mov: Move, _timer: &Timer) {}
    fn make_move(&mut self, position: &Position, timer: &Timer) -> Move;
}
