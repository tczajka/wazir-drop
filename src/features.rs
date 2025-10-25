use std::iter;

use crate::{
    either::Either, enums::EnumMap, Color, Move, NormalizedSquare, Piece, Position, RegularMove,
    SetupMove,
};

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
    ) -> Option<(impl Iterator<Item = usize>, impl Iterator<Item = usize>)> {
        match mov {
            Move::Setup(mov) => Self::diff_setup(position, mov, color)
                .map(|(added, removed)| (Either::Left(added), Either::Left(removed))),
            Move::Regular(mov) => Self::diff_regular(position, mov, color)
                .map(|(added, removed)| (Either::Right(added), Either::Right(removed))),
        }
    }

    fn diff_setup(
        position: &Position,
        mov: SetupMove,
        color: Color,
    ) -> Option<(impl Iterator<Item = usize>, impl Iterator<Item = usize>)>;

    fn diff_regular(
        position: &Position,
        mov: RegularMove,
        color: Color,
    ) -> Option<(impl Iterator<Item = usize>, impl Iterator<Item = usize>)>;
}

enum PieceSquareFeatures {}

impl Features for PieceSquareFeatures {
    const COUNT: usize = NormalizedSquare::COUNT + NUM_CAPTURED_INDEXES;

    fn all(position: &Position, color: Color) -> impl Iterator<Item = usize> {
        // TODO: Implement.
        iter::empty()
    }

    fn diff_setup(
        position: &Position,
        mov: SetupMove,
        color: Color,
    ) -> Option<(impl Iterator<Item = usize>, impl Iterator<Item = usize>)> {
        // TODO: Implement.
        Some((iter::empty(), iter::empty()))
    }

    fn diff_regular(
        position: &Position,
        mov: RegularMove,
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
