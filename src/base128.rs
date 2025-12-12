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
pub const VARINT_BASE_BITS: u32 = 5;
pub const VARINT_EXTENSION_BITS: u32 = 2;

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

    pub fn encode_varint(&mut self, n: i32) {
        let (sign_bit, mut val) = if n < 0 {
            (1, (-(n + 1)) as u32)
        } else {
            (0, n as u32)
        };
        self.encode_bits(1, sign_bit);
        self.encode_bits(VARINT_BASE_BITS, val & ((1 << VARINT_BASE_BITS) - 1));
        val >>= VARINT_BASE_BITS;
        while val != 0 {
            self.encode_bits(1, 1);
            self.encode_bits(
                VARINT_EXTENSION_BITS,
                val & ((1 << VARINT_EXTENSION_BITS) - 1),
            );
            val >>= VARINT_EXTENSION_BITS;
        }
        self.encode_bits(1, 0);
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
