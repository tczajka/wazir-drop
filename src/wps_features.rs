use crate::{
    captured_index, enums::SimpleEnumExt, linear_wps_weights, smallvec::SmallVec, Color, Features,
    LinearEvaluator, NormalizedSquare, Piece, Position, RegularMove, SetupMove, Square, Symmetry,
    NUM_CAPTURED_INDEXES,
};
use std::iter;

/// Wazir-Piece-Square features.
#[derive(Debug, Clone, Copy)]
pub struct WPSFeatures;

impl WPSFeatures {
    const CAPTURED_OFFSET: usize = (2 * Piece::COUNT - 1) * Square::COUNT;
    const COUNT_PER_WAZIR: usize = Self::CAPTURED_OFFSET + 2 * (NUM_CAPTURED_INDEXES - 2);
    const COUNT: usize = NormalizedSquare::COUNT * Self::COUNT_PER_WAZIR;

    pub fn board_feature(
        wazir_square: NormalizedSquare,
        is_other_color: bool,
        piece: Piece,
        square: Square,
    ) -> usize {
        wazir_square.index() * Self::COUNT_PER_WAZIR
            + (usize::from(is_other_color) * (Piece::COUNT - 1) + piece.index()) * Square::COUNT
            + square.index()
    }

    pub fn captured_feature(
        wazir_square: NormalizedSquare,
        is_other_color: bool,
        piece: Piece,
        index: usize,
    ) -> usize {
        wazir_square.index() * Self::COUNT_PER_WAZIR
            + Self::CAPTURED_OFFSET
            + usize::from(is_other_color) * (NUM_CAPTURED_INDEXES - 2)
            + captured_index(piece, index)
    }
}

impl Features for WPSFeatures {
    fn count(self) -> usize {
        Self::COUNT
    }

    fn all(self, position: &Position, color: Color) -> impl Iterator<Item = usize> {
        position
            .occupied_by_piece(Piece::Wazir.with_color(color))
            .into_iter()
            .flat_map(move |wazir_square| {
                let (symmetry, wazir_nsquare) = Symmetry::normalize(wazir_square);
                [(false, color), (true, color.opposite())]
                    .into_iter()
                    .flat_map(move |(is_other_color, other_color)| {
                        Piece::all()
                            .filter(move |&piece| (piece, is_other_color) != (Piece::Wazir, false))
                            .flat_map(move |piece| {
                                let cpiece = piece.with_color(other_color);
                                let num_captured = if piece == Piece::Wazir {
                                    0
                                } else {
                                    position.num_captured(cpiece)
                                };
                                position
                                    .occupied_by_piece(cpiece)
                                    .into_iter()
                                    .map(move |square| {
                                        Self::board_feature(
                                            wazir_nsquare,
                                            is_other_color,
                                            piece,
                                            symmetry.apply(square),
                                        )
                                    })
                                    .chain((0..num_captured).map(move |index| {
                                        Self::captured_feature(
                                            wazir_nsquare,
                                            is_other_color,
                                            piece,
                                            index,
                                        )
                                    }))
                            })
                    })
            })
    }

    fn diff_setup(
        self,
        mov: SetupMove,
        new_position: &Position,
        color: Color,
    ) -> Option<(impl Iterator<Item = usize>, impl Iterator<Item = usize>)> {
        if (color, mov.color) != (Color::Red, Color::Blue) {
            return None;
        }
        let wazir_square = new_position
            .occupied_by_piece(Piece::Wazir.with_color(color))
            .first()
            .unwrap();
        let (symmetry, wazir_nsquare) = Symmetry::normalize(wazir_square);
        let mov_symmetry = Symmetry::pov(mov.color);
        let added = mov
            .pieces
            .into_iter()
            .enumerate()
            .map(move |(index, piece)| {
                let square = mov_symmetry.apply(Square::from_index(index));
                Self::board_feature(wazir_nsquare, true, piece, symmetry.apply(square))
            });
        Some((added, iter::empty()))
    }

    fn diff_regular(
        self,
        mov: RegularMove,
        new_position: &Position,
        color: Color,
    ) -> Option<(impl Iterator<Item = usize>, impl Iterator<Item = usize>)> {
        if mov.colored_piece == Piece::Wazir.with_color(color)
            || mov.colored_piece.color() != color && mov.captured == Some(Piece::Wazir)
        {
            return None;
        }
        let wazir_square = new_position
            .occupied_by_piece(Piece::Wazir.with_color(color))
            .first()
            .unwrap();
        let (symmetry, wazir_nsquare) = Symmetry::normalize(wazir_square);

        let mov_color = mov.colored_piece.color();
        let piece = mov.colored_piece.piece();
        let is_opp_move = mov_color != color;

        let mut added: SmallVec<usize, 2> = SmallVec::new();
        let mut removed: SmallVec<usize, 2> = SmallVec::new();

        let source = match mov.from {
            Some(from) => {
                Self::board_feature(wazir_nsquare, is_opp_move, piece, symmetry.apply(from))
            }
            None => {
                // Note: This is a drop, so we're not capturing the same piece again.
                Self::captured_feature(
                    wazir_nsquare,
                    is_opp_move,
                    piece,
                    new_position.num_captured(mov.colored_piece),
                )
            }
        };
        removed.push(source);
        added.push(Self::board_feature(
            wazir_nsquare,
            is_opp_move,
            piece,
            symmetry.apply(mov.to),
        ));
        if let Some(captured_piece) = mov.captured {
            removed.push(Self::board_feature(
                wazir_nsquare,
                !is_opp_move,
                captured_piece,
                symmetry.apply(mov.to),
            ));
            if captured_piece != Piece::Wazir {
                // This is a capture, so we didn't drop the same piece.
                added.push(Self::captured_feature(
                    wazir_nsquare,
                    is_opp_move,
                    captured_piece,
                    new_position.num_captured(captured_piece.with_color(mov_color)) - 1,
                ));
            }
        }
        Some((added.into_iter(), removed.into_iter()))
    }
}

impl Default for LinearEvaluator<WPSFeatures> {
    fn default() -> Self {
        Self::new(
            WPSFeatures,
            linear_wps_weights::TO_MOVE,
            &linear_wps_weights::FEATURES,
        )
    }
}
