use extra::moverand;
use rand::Rng;
use wazir_drop::{AnyMove, Position, Stage};

pub fn random_opening<RNG: Rng>(len: usize, rng: &mut RNG) -> Vec<AnyMove> {
    let mut moves = Vec::new();
    let mut position = Position::initial();
    while moves.len() < len && !matches!(position.stage(), Stage::End(_)) {
        let mov = moverand::random_move(&position, rng);
        position = position.make_any_move(mov).unwrap();
        moves.push(mov);
    }
    moves
}
