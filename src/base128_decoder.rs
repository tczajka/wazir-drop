use std::str::Chars;

/// 2-byte, 11-bit character (special << 4) + x encodes sequence SPECIAL_MAP[special], x
pub static SPECIAL_MAP: [Option<u8>; 16] = [
    None,       // Can't use 0 because that would be overlong encoding.
    None,       // Avoid 1 to skip control codes U+80..U+A0.
    Some(0),    // NUL
    Some(8),    // Backspace
    Some(9),    // Tab
    Some(10),   // LF
    Some(11),   // VT
    Some(12),   // FF
    Some(13),   // CR
    Some(27),   // ESC
    Some(b'"'), // Quotation mark
    None,
    None, // Avoid 12 to skip U+61C ARABIC LETTER MARK that causes right-to-left issues.
    None,
    None,
    None,
];

// Varints are encoded as: sign bit, BASE_BITS, extension bit, EXTENSION_BITS, extension bit, EXTENSION_BITS, ...
pub const VARINT_BASE_BITS: u32 = 6;
pub const VARINT_EXTENSION_BITS: u32 = 3;

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

    pub fn decode_varint(&mut self) -> i32 {
        let sign = self.decode_bits(1);
        let mut value = self.decode_bits(VARINT_BASE_BITS);
        let mut shift = VARINT_BASE_BITS;
        while self.decode_bits(1) != 0 {
            let ext = self.decode_bits(VARINT_EXTENSION_BITS);
            value |= ext << shift;
            shift += VARINT_EXTENSION_BITS;
        }
        if sign != 0 {
            -(value as i32) - 1
        } else {
            value as i32
        }
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
        } else if special < 16 {
            let special = SPECIAL_MAP[usize::try_from(special).unwrap()];
            let special = special.expect("Invalid special base128 code");
            let special = u32::from(special);
            (14, special | bits << 7)
        } else {
            panic!("Invalid base128 character");
        }
    }
}
