use crate::{
    base128::{Base128Decoder, Base128Encoder},
    Color, Piece, SetupMove,
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
