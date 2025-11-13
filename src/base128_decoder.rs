use std::str::Chars;

pub struct Base128Decoder<'a> {
    input: Chars<'a>,
    // 0..=14
    num_buffered_bits: u32,
    // 0..(1 << num_buffered_bits)
    buffered_bits: u64,
}

impl<'a> Base128Decoder<'a> {
    pub fn new(s: &'a str) -> Self {
        Self {
            input: s.chars(),
            num_buffered_bits: 0,
            buffered_bits: 0,
        }
    }

    pub fn decode_bits(&mut self, n: u32) -> u32 {
        assert!(n <= 32);
        while self.num_buffered_bits < n {
            let c = self.input.next().expect("Unxpected end of base128");
            let (k, bits) = Self::decode_char(c);
            self.buffered_bits |= u64::from(bits) << self.num_buffered_bits;
            self.num_buffered_bits += k;
        }
        let res = (self.buffered_bits & ((1 << n) - 1)) as u32;
        self.buffered_bits >>= n;
        self.num_buffered_bits -= n;
        res
    }

    /// Panics if the stream is not finished properly.
    pub fn finish(mut self) {
        if self.decode_bits(1) != 1 || self.buffered_bits != 0 || self.input.next().is_some() {
            panic!("Expected end of base128");
        }
    }

    // num bits, bits
    fn decode_char(c: char) -> (u32, u32) {
        let c = u32::from(c);
        let bits = c & 0x7F;
        let special = c >> 7;
        if special == 0 {
            (7, bits)
        } else {
            let special = u32::from(Self::decode_special(special));
            (14, special | bits << 7)
        }
    }

    fn decode_special(special: u32) -> u8 {
        match special {
            2 => 0,
            3 => 8,
            4 => 9,
            5 => 10,
            6 => 11,
            7 => 12,
            8 => 13,
            9 => 27,
            10 => b'"',
            _ => panic!("Invalid base128 character"),
        }
    }
}
