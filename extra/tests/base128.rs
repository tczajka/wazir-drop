use extra::base128_encoder::Base128Encoder;
use rand::{Rng, SeedableRng, rngs::StdRng};
use wazir_drop::base128_decoder::Base128Decoder;

#[test]
fn test_base128() {
    let mut rng = StdRng::from_os_rng();
    for _ in 0..100 {
        let mut bytes = Vec::new();
        let mut encoder = Base128Encoder::new();
        for _ in 0..10000 {
            let byte: u8 = rng.random();
            bytes.push(byte);
            let b: u32 = byte.into();
            encoder.encode_bits(8, b);
        }
        let s = encoder.finish();

        let mut decoder = Base128Decoder::new(&s);
        for &byte in &bytes {
            let b = decoder.decode_bits(8);
            assert_eq!(b, u32::from(byte));
        }
        decoder.finish();
    }
}

#[test]
fn test_varint() {
    let numbers = [
        i32::MIN,
        i32::MAX,
        -1000000,
        -100,
        -5,
        -1,
        0,
        1,
        5,
        100,
        1000000,
    ];
    let mut encoder = Base128Encoder::new();
    for &n in &numbers {
        encoder.encode_varint(n);
    }
    let s = encoder.finish();
    let mut decoder = Base128Decoder::new(&s);
    for &n in &numbers {
        let x = decoder.decode_varint();
        assert_eq!(x, n);
    }
    decoder.finish();
}
