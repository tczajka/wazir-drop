use std::iter;

use crate::{
    enums::{EnumMap, SimpleEnumExt},
    smallvec::SmallVec,
    Bitboard, Color, InvalidMove, Piece, Position, RegularMove, SetupMove, Square, Stage,
};

static MOVE_BITBOARD_TABLE: EnumMap<Piece, EnumMap<Square, Bitboard>> = calc_move_bitboard_table();

pub fn move_bitboard(piece: Piece, square: Square) -> Bitboard {
    MOVE_BITBOARD_TABLE[piece][square]
}

pub fn validate_from_to(piece: Piece, from: Square, to: Square) -> Result<(), InvalidMove> {
    if !move_bitboard(piece, from).contains(to) {
        return Err(InvalidMove);
    }
    Ok(())
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

pub fn setup_moves(color: Color) -> impl Iterator<Item = SetupMove> {
    SetupMoveIterator { color, mov: None }
}

#[derive(Debug)]
struct SetupMoveIterator {
    color: Color,
    mov: Option<SetupMove>,
}

impl Iterator for SetupMoveIterator {
    type Item = SetupMove;

    fn next(&mut self) -> Option<Self::Item> {
        match self.mov {
            None => {
                let pieces: SmallVec<Piece, { SetupMove::SIZE }> = Piece::all()
                    .flat_map(|piece| iter::repeat_n(piece, piece.initial_count()))
                    .collect();
                let pieces = (&pieces[..]).try_into().unwrap();
                self.mov = Some(SetupMove {
                    color: self.color,
                    pieces,
                });
            }
            Some(ref mut mov) => {
                let mut i = SetupMove::SIZE - 1;
                loop {
                    // mov.pieces[i..] is in non-ascending order
                    if i == 0 {
                        return None;
                    }
                    i -= 1;
                    if mov.pieces[i] < mov.pieces[i + 1] {
                        break;
                    }
                }
                // mov.pieces[i] < mov.pieces[i+1] >= ...
                let mut j = i + 1;
                while j != SetupMove::SIZE - 1 && mov.pieces[i] < mov.pieces[j + 1] {
                    j += 1;
                }
                // mov.pieces[i] < mov.pieces[j]
                // mov.pieces[i] >= mov.pieces[j+1]
                mov.pieces.swap(i, j);
                mov.pieces[i + 1..].reverse();
                self.mov = Some(*mov);
            }
        }
        self.mov
    }
}

// Generate all pseudomoves.
// Includes non-escapes and suicides.
pub fn pseudomoves(position: Position) -> impl Iterator<Item = RegularMove> {
    captures(position)
        .chain(pseudojumps(position))
        .chain(drops(position))
}

// Generate all captures
// If in check, includes non-escapes.
pub fn captures(position: Position) -> impl Iterator<Item = RegularMove> {
    assert!(position.stage() == Stage::Regular);
    let me = position.to_move();
    let opp = me.opposite();
    let opp_mask = position.occupied_by(opp);
    iter::empty()
    // TODO: Implement.
}

// Generate all pseudojumps (not captures).
// Includes non-escapes and suicides.
pub fn pseudojumps(position: Position) -> impl Iterator<Item = RegularMove> {
    // TODO: Implement.
    iter::empty()
}

// Piece drops.
// If in check, these are non-escapes.
pub fn drops(position: Position) -> impl Iterator<Item = RegularMove> {
    // TODO: Implement.
    iter::empty()
}
