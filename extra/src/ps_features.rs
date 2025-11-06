use std::{fmt::Debug, iter};
use wazir_drop::{
    Color, Features, LinearEvaluator, NUM_CAPTURED_INDEXES, NormalizedSquare, Piece, Position,
    RegularMove, SetupMove, Square, Symmetry, captured_index, enums::SimpleEnumExt,
    smallvec::SmallVec,
};

use crate::linear_ps_weights;

/// Piece-Square features.
#[derive(Debug, Clone, Copy)]
pub struct PSFeatures;

impl PSFeatures {
    const CAPTURED_OFFSET: usize = Piece::COUNT * NormalizedSquare::COUNT;

    pub fn board_feature(piece: Piece, normalized_square: NormalizedSquare) -> usize {
        piece.index() * NormalizedSquare::COUNT + normalized_square.index()
    }

    fn board_feature_unnormalized(piece: Piece, square: Square) -> usize {
        let (_, normalized_square) = Symmetry::normalize(square);
        Self::board_feature(piece, normalized_square)
    }

    pub fn captured_feature(piece: Piece, index: usize) -> usize {
        Self::CAPTURED_OFFSET + captured_index(piece, index)
    }
}

impl Features for PSFeatures {
    fn count(self) -> usize {
        Self::CAPTURED_OFFSET + NUM_CAPTURED_INDEXES - 2
    }

    fn all(self, position: &Position, color: Color) -> impl Iterator<Item = usize> {
        Piece::all()
            .flat_map(move |piece| {
                position
                    .occupied_by_piece(piece.with_color(color))
                    .into_iter()
                    .map(move |square| Self::board_feature_unnormalized(piece, square))
            })
            .chain(Piece::all_non_wazir().flat_map(move |piece| {
                let offset = Self::captured_feature(piece, 0);
                (0..position.num_captured(piece.with_color(color))).map(move |index| offset + index)
            }))
    }

    fn diff_setup(
        self,
        mov: SetupMove,
        _new_position: &Position,
        color: Color,
    ) -> Option<(impl Iterator<Item = usize>, impl Iterator<Item = usize>)> {
        let mut added: SmallVec<usize, { SetupMove::SIZE }> = SmallVec::new();
        if mov.color == color {
            let symmetry = Symmetry::pov(color);
            for (index, &piece) in mov.pieces.iter().enumerate() {
                let square = symmetry.apply(Square::from_index(index));
                added.push(Self::board_feature_unnormalized(piece, square));
            }
        };
        Some((added.into_iter(), iter::empty()))
    }

    fn diff_regular(
        self,
        mov: RegularMove,
        new_position: &Position,
        color: Color,
    ) -> Option<(impl Iterator<Item = usize>, impl Iterator<Item = usize>)> {
        // piece, captured
        let mut added: SmallVec<usize, 2> = SmallVec::new();
        let mut removed: Option<usize> = None;
        if mov.colored_piece.color() == color {
            let piece = mov.colored_piece.piece();
            match mov.from {
                Some(from) => {
                    removed = Some(Self::board_feature_unnormalized(piece, from));
                }
                None => {
                    // Note: This is a drop, so we're not capturing the same piece again.
                    removed = Some(Self::captured_feature(
                        piece,
                        new_position.num_captured(mov.colored_piece),
                    ));
                }
            }
            added.push(Self::board_feature_unnormalized(piece, mov.to));
            if let Some(captured_piece) = mov.captured
                && captured_piece != Piece::Wazir
            {
                // This is a capture, so we didn't drop the same piece.
                added.push(Self::captured_feature(
                    captured_piece,
                    new_position.num_captured(captured_piece.with_color(color)) - 1,
                ));
            }
        } else if let Some(captured_piece) = mov.captured {
            removed = Some(Self::board_feature_unnormalized(captured_piece, mov.to));
        }
        Some((added.into_iter(), removed.into_iter()))
    }
}

pub fn default_linear_ps_features() -> LinearEvaluator<PSFeatures> {
    LinearEvaluator::new(
        PSFeatures,
        linear_ps_weights::TO_MOVE,
        &linear_ps_weights::FEATURES,
    )
}
