use std::iter;

use crate::{enums::EnumMap, Color, Move, NormalizedSquare, Piece, Position, SetupMove, Square};

pub trait Features {
    const COUNT: usize;

    fn all(position: &Position, color: Color) -> impl Iterator<Item = usize>;

    /// Returns (added features, removed features).
    ///
    /// If it's too complicated, returns `None`. Caller should fall back to `all_features`.
    fn diff(
        position: &Position,
        mov: Move,
        color: Color,
    ) -> Option<(impl Iterator<Item = usize>, impl Iterator<Item = usize>)>;
}

struct PieceSquareFeatures;

impl Features for PieceSquareFeatures {
    const COUNT: usize = NormalizedSquare::COUNT + NUM_CAPTURED_INDEXES;

    fn all(position: &Position, color: Color) -> impl Iterator<Item = usize> {
        // TODO: Implement.
        iter::empty()
    }

    fn diff(
        position: &Position,
        mov: Move,
        color: Color,
    ) -> Option<(impl Iterator<Item = usize>, impl Iterator<Item = usize>)> {
        // TODO: Implement.
        Some((iter::empty(), iter::empty()))
    }
}

/// Not counting Wazirs.
const NUM_CAPTURED_INDEXES: usize = Color::COUNT * (SetupMove::SIZE - 1);

fn captured_index(piece: Piece, index: usize) -> usize {
    CAPTURED_OFFSET_TABLE[piece] + index
}

static CAPTURED_OFFSET_TABLE: EnumMap<Piece, usize> = {
    let mut table = [0; Piece::COUNT];
    let mut sum = 0;
    let mut piece_idx = 0;
    while piece_idx != Piece::COUNT {
        table[piece_idx] = sum;
        sum += Color::COUNT * Piece::from_index(piece_idx).initial_count();
        piece_idx += 1;
    }
    assert!(table[Piece::Wazir.index()] == NUM_CAPTURED_INDEXES);
    EnumMap::from_array(table)
};
