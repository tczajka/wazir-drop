use wazir_drop::{
    base128::{Base128Decoder, Base128Encoder},
    book::{decode_setup_move, encode_setup_move},
    movegen, Color,
};

#[test]
fn test_encode_setup_move() {
    let mut encoder = Base128Encoder::new();
    let setup_move = movegen::setup_moves(Color::Red).next().unwrap();
    encode_setup_move(&mut encoder, setup_move);
    let encoded = encoder.finish();

    let mut decoder = Base128Decoder::new(&encoded);
    let mov = decode_setup_move(&mut decoder, Color::Red);
    decoder.finish();
    assert_eq!(mov, setup_move);
}
