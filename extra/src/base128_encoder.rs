use wazir_drop::base128_decoder::SPECIAL_MAP;

/// Encodes a sequence of bits into a valid UTF-8 encoded String.
/// n bits get converted to n/7 bytes.
pub struct Base128Encoder {
    // The resulting String.
    output: String,
    /// Top 4 bits of a 2-byte, 11-bit codepoint.
    /// 110xxxxx 10xxxxxx
    /// Must be in the range 1..16.
    special: Option<u32>,
    /// 0..7
    num_buffered_bits: u32,
    /// 0..(1 << num_buffered_bits)
    buffered_bits: u64,
}

impl Base128Encoder {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            special: None,
            num_buffered_bits: 0,
            buffered_bits: 0,
        }
    }

    pub fn encode_bits(&mut self, n: u32, bits: u32) {
        assert!(n == 32 || n < 32 && bits < 1 << n);
        self.buffered_bits |= u64::from(bits) << self.num_buffered_bits;
        self.num_buffered_bits += n;

        while self.num_buffered_bits >= 7 {
            let ascii = (self.buffered_bits & 0x7F) as u8;
            self.buffered_bits >>= 7;
            self.num_buffered_bits -= 7;
            self.push_ascii(ascii);
        }
    }

    pub fn finish(mut self) -> String {
        self.encode_bits(1, 1);
        if self.num_buffered_bits != 0 {
            self.encode_bits(7 - self.num_buffered_bits, 0);
        }
        if self.special.is_some() {
            self.encode_bits(7, 0);
        }
        assert_eq!(self.num_buffered_bits, 0);
        assert!(self.special.is_none());
        self.output
    }

    fn push_ascii(&mut self, ascii: u8) {
        match self.special {
            None => match Self::ascii_to_special(ascii) {
                None => self.output.push(ascii.into()),
                Some(special) => self.special = Some(special),
            },
            Some(special) => {
                let c = special << 7 | u32::from(ascii);
                self.output.push(c.try_into().unwrap());
                self.special = None;
            }
        }
    }

    fn ascii_to_special(ascii: u8) -> Option<u32> {
        SPECIAL_MAP
            .iter()
            .enumerate()
            .find(|&(_, &a)| a == Some(ascii))
            .map(|(i, _)| i.try_into().unwrap())
    }
}
