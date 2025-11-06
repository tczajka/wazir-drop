use crate::{NormalizedSquare, Piece, Square, NUM_CAPTURED_INDEXES};

#[derive(Debug, Clone, Copy)]
pub struct WPSFeatures;

impl WPSFeatures {
    const CAPTURED_OFFSET_PER_WAZIR: usize = (2 * Piece::COUNT - 1) * Square::COUNT;
    const COUNT_PER_WAZIR: usize = Self::CAPTURED_OFFSET_PER_WAZIR + NUM_CAPTURED_INDEXES - 2;

    pub fn board_feature(
        wazir_pos: NormalizedSquare,
        other_color: bool,
        piece: Piece,
        square: Square,
    ) -> usize {
        wazir_pos.index() * Self::COUNT_PER_WAZIR
            + (usize::from(other_color) * (Piece::COUNT - 1) + piece.index()) * Square::COUNT
            + square.index()
    }
}
