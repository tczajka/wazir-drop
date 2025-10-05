use crate::{enums::EnumMap, Bitboard, Piece, Square};

static MOVE_BITBOARD_TABLE: EnumMap<Piece, EnumMap<Square, Bitboard>> = calc_move_bitboard_table();

pub fn move_bitboard(piece: Piece, square: Square) -> Bitboard {
    MOVE_BITBOARD_TABLE[piece][square]
}

const fn calc_move_bitboard_table() -> EnumMap<Piece, EnumMap<Square, Bitboard>> {
    let mut table = [EnumMap::from_array([Bitboard::EMPTY; Square::COUNT]); Piece::COUNT];
    let mut piece_idx = 0;
    while piece_idx != Piece::COUNT {
        table[piece_idx] = calc_move_bitboard_table_for_piece(Piece::from_index(piece_idx));
        piece_idx += 1;
    }
    EnumMap::from_array(table)
}

const fn calc_move_bitboard_table_for_piece(piece: Piece) -> EnumMap<Square, Bitboard> {
    let mut table = [Bitboard::EMPTY; Square::COUNT];
    let mut square_idx = 0;
    while square_idx != Square::COUNT {
        table[square_idx] = calc_move_bitboard(piece, Square::from_index(square_idx));
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
            bitboard = bitboard.with_square(square2);
        }
        i += 1;
    }
    bitboard
}
