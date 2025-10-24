use rand::Rng;
use wazir_drop::{Move, Position, Stage, movegen};

pub fn random_opening<RNG: Rng>(len: usize, rng: &mut RNG) -> Vec<Move> {
    let mut moves = Vec::new();
    let mut position = Position::initial();
    while moves.len() < len && position.stage() != Stage::End {
        let mov = movegen::random_move(&position, rng);
        position = position.make_move(mov).unwrap();
        moves.push(mov);
    }
    moves
}
