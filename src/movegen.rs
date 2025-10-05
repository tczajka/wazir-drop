use crate::{
    enums::{EnumMap, SimpleEnumExt},
    Bitboard, Piece, Square,
};
use std::mem;

static MOVE_BITBOARD_TABLE: EnumMap<Piece, EnumMap<Square, Bitboard>> = calc_move_bitboard_table();

pub fn move_bitboard(piece: Piece, square: Square) -> Bitboard {
    MOVE_BITBOARD_TABLE[piece][square]
}

const fn calc_move_bitboard_table() -> EnumMap<Piece, EnumMap<Square, Bitboard>> {
    let mut table = [EnumMap::from_array([Bitboard::EMPTY; Square::COUNT]); Piece::COUNT];
    let mut piece_idx = 0;
    while piece_idx != Piece::COUNT {
        let piece: Piece = unsafe { mem::transmute(piece_idx as u8) };
        table[piece_idx] = calc_move_bitboard_table_for_piece(piece);
        piece_idx += 1;
    }
    EnumMap::from_array(table)
}

const fn calc_move_bitboard_table_for_piece(piece: Piece) -> EnumMap<Square, Bitboard> {
    let mut table = [Bitboard::EMPTY; Square::COUNT];
    let mut square_idx = 0;
    while square_idx != Square::COUNT {
        let square: Square = unsafe { mem::transmute(square_idx as u8) };
        table[square_idx] = calc_move_bitboard(piece, square);
        square_idx += 1;
    }
    EnumMap::from_array(table)
}

const fn calc_move_bitboard(piece: Piece, square: Square) -> Bitboard {
    let mut bitboard = Bitboard::EMPTY;
    let directions = piece.directions();
    let mut i = 0;
    while i != directions.len() {
        if let Some(square2) = square.add(directions[i]) {
            bitboard = bitboard.or(Bitboard::single(square2));
        }
        i += 1;
    }
    bitboard
}
