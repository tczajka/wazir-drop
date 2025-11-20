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
        eprintln!("base128: {s}");

        let mut decoder = Base128Decoder::new(&s);
        for &byte in &bytes {
            let b = decoder.decode_bits(8);
            assert_eq!(b, u32::from(byte));
        }
        decoder.finish();
    }
}
