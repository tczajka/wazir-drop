use crate::{captured_index, Features, NormalizedSquare, Piece, Square, NUM_CAPTURED_INDEXES};

#[derive(Debug, Clone, Copy)]
pub struct WPSFeatures;

impl WPSFeatures {
    const CAPTURED_OFFSET: usize = (2 * Piece::COUNT - 1) * Square::COUNT;
    const COUNT_PER_WAZIR: usize = Self::CAPTURED_OFFSET + 2 * (NUM_CAPTURED_INDEXES - 2);
    const COUNT: usize = NormalizedSquare::COUNT * Self::COUNT_PER_WAZIR;

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

    pub fn captured_feature(
        wazir_pos: NormalizedSquare,
        other_color: bool,
        piece: Piece,
        index: usize,
    ) -> usize {
        wazir_pos.index() * Self::COUNT_PER_WAZIR
            + Self::CAPTURED_OFFSET
            + usize::from(other_color) * (NUM_CAPTURED_INDEXES - 2)
            + captured_index(piece, index)
    }
}

/*
impl Features for WPSFeatures {
    fn count(self) -> usize {
        Self::COUNT
    }
}
*/
