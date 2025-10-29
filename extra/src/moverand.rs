use rand::{
    Rng,
    seq::{IteratorRandom, SliceRandom},
};
use wazir_drop::{Color, Move, Position, RegularMove, SetupMove, Stage, movegen};

pub fn random_setup<RNG: Rng>(color: Color, rng: &mut RNG) -> SetupMove {
    let mut mov = movegen::setup_moves(color).next().unwrap();
    mov.pieces.shuffle(rng);
    mov
}

pub fn random_regular<RNG: Rng>(position: &Position, rng: &mut RNG) -> RegularMove {
    movegen::regular_pseudomoves(position)
        .choose(rng)
        .expect("Stalemate")
}

pub fn random_move<RNG: rand::Rng>(position: &Position, rng: &mut RNG) -> Move {
    match position.stage() {
        Stage::Setup => Move::Setup(random_setup(position.to_move(), rng)),
        Stage::Regular => Move::Regular(random_regular(position, rng)),
        Stage::End(_) => panic!("End of game"),
    }
}
