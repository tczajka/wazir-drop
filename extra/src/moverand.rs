use rand::{
    Rng,
    seq::{IteratorRandom, SliceRandom},
};
use wazir_drop::{AnyMove, Color, Move, Position, SetupMove, Stage, movegen};

pub fn random_setup<RNG: Rng>(color: Color, rng: &mut RNG) -> SetupMove {
    let mut mov = movegen::setup_moves(color).next().unwrap();
    mov.pieces.shuffle(rng);
    mov
}

pub fn random_regular<RNG: Rng>(position: &Position, rng: &mut RNG) -> Move {
    movegen::pseudomoves(position)
        .choose(rng)
        .expect("Stalemate")
}

pub fn random_move<RNG: rand::Rng>(position: &Position, rng: &mut RNG) -> AnyMove {
    match position.stage() {
        Stage::Setup => AnyMove::Setup(random_setup(position.to_move(), rng)),
        Stage::Regular => AnyMove::Regular(random_regular(position, rng)),
        Stage::End(_) => panic!("End of game"),
    }
}
