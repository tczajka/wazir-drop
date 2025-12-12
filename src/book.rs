use std::time::{Duration, SystemTime};

use crate::{
    base128::{Base128Decoder, Base128Encoder},
    book_data,
    constants::{RED_SETUP_INDEX_BEGIN, RED_SETUP_INDEX_END},
    log, Color, Piece, SetupMove, Symmetry,
};

pub fn encode_setup_move(encoder: &mut Base128Encoder, setup_move: SetupMove) {
    for &piece in &setup_move.pieces {
        encode_piece(encoder, piece);
    }
}

pub fn decode_setup_move(decoder: &mut Base128Decoder, color: Color) -> SetupMove {
    let mut pieces = [Piece::Alfil; SetupMove::SIZE];
    for piece in &mut pieces {
        *piece = decode_piece(decoder);
    }
    SetupMove { color, pieces }
}

pub fn encode_piece(encoder: &mut Base128Encoder, piece: Piece) {
    match piece {
        Piece::Alfil => encoder.encode_bits(1, 0b0),
        Piece::Dabbaba => encoder.encode_bits(2, 0b01),
        Piece::Ferz => encoder.encode_bits(3, 0b011),
        Piece::Knight => encoder.encode_bits(4, 0b0111),
        Piece::Wazir => encoder.encode_bits(4, 0b1111),
    }
}

pub fn decode_piece(decoder: &mut Base128Decoder) -> Piece {
    if decoder.decode_bits(1) == 0 {
        Piece::Alfil
    } else if decoder.decode_bits(1) == 0 {
        Piece::Dabbaba
    } else if decoder.decode_bits(1) == 0 {
        Piece::Ferz
    } else if decoder.decode_bits(1) == 0 {
        Piece::Knight
    } else {
        Piece::Wazir
    }
}

pub fn red_setup() -> SetupMove {
    let r = time_based_random(2 * (RED_SETUP_INDEX_END - RED_SETUP_INDEX_BEGIN));
    let setup_idx = r / 2 + RED_SETUP_INDEX_BEGIN;
    let symmetry = [Symmetry::Identity, Symmetry::FlipX][r % 2];
    log::info!("red setup #{setup_idx} {symmetry}");
    for book_opening in BookIterator::new() {
        if book_opening.index == setup_idx {
            return symmetry.apply_to_setup(book_opening.red);
        }
    }
    panic!("red opening not found")
}

fn time_based_random(n: usize) -> usize {
    // Compute hash x = (a * t + b) % MODULUS % n;
    const MODULUS: u128 = (1 << 61) - 1;
    const A: u128 = 0x10c82ee50ad34876;
    const B: u128 = 0x41cd6910f455faa;
    let t: u128 = SystemTime::UNIX_EPOCH
        .elapsed()
        .unwrap_or(Duration::ZERO)
        .as_nanos()
        % MODULUS;
    let x = (A * t + B) % MODULUS % n as u128;
    x as usize
}

pub fn blue_setup(red: SetupMove) -> Option<SetupMove> {
    let (symmetry, red) = Symmetry::normalize_red_setup(red);
    for book_opening in BookIterator::new() {
        if book_opening.red == red {
            log::info!("blue setup #{index}", index = book_opening.index);
            return Some(symmetry.inverse().apply_to_setup(book_opening.blue));
        }
    }
    log::info!("opening not found");
    None
}

pub fn blue_setup_moves() -> Vec<SetupMove> {
    BookIterator::new()
        .flat_map(|book_opening| {
            let mov = book_opening.blue.with_color(Color::Blue);
            [Symmetry::Identity, Symmetry::FlipX]
                .iter()
                .map(move |symmetry| symmetry.apply_to_setup(mov))
        })
        .collect()
}

struct BookOpening {
    index: usize,
    red: SetupMove,
    blue: SetupMove,
}

struct BookIterator {
    next_index: usize,
    decoder: Option<Base128Decoder<'static>>,
}

impl BookIterator {
    fn new() -> Self {
        Self {
            next_index: 0,
            decoder: Some(Base128Decoder::new(book_data::OPENINGS)),
        }
    }
}

impl Iterator for BookIterator {
    type Item = BookOpening;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_index >= book_data::NUM_OPENINGS {
            if let Some(decoder) = self.decoder.take() {
                decoder.finish();
            }
            return None;
        }
        let decoder = self.decoder.as_mut().unwrap();
        let red = decode_setup_move(decoder, Color::Red);
        let blue = decode_setup_move(decoder, Color::Blue);
        let opening = BookOpening {
            index: self.next_index,
            red,
            blue,
        };
        self.next_index += 1;
        Some(opening)
    }
}
