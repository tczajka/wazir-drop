use crate::{
    captured_index, either::Either, enums::SimpleEnumExt, smallvec::SmallVec, Color, Move,
    NormalizedSquare, Piece, Position, RegularMove, SetupMove, Square, Symmetry,
    NUM_CAPTURED_INDEXES,
};
use std::{fmt::Debug, iter};

pub trait Features: Debug + Copy + Send + 'static {
    fn count(self) -> usize;

    fn all(self, position: &Position, color: Color) -> impl Iterator<Item = usize>;

    /// Returns (added features, removed features).
    ///
    /// If it's too complicated, returns `None`. Caller should fall back to `all_features`.
    fn diff(
        self,
        mov: Move,
        new_position: &Position,
        color: Color,
    ) -> Option<(impl Iterator<Item = usize>, impl Iterator<Item = usize>)> {
        match mov {
            Move::Setup(mov) => self
                .diff_setup(mov, new_position, color)
                .map(|(added, removed)| (Either::Left(added), Either::Left(removed))),
            Move::Regular(mov) => self
                .diff_regular(mov, new_position, color)
                .map(|(added, removed)| (Either::Right(added), Either::Right(removed))),
        }
    }

    fn diff_setup(
        self,
        mov: SetupMove,
        new_position: &Position,
        color: Color,
    ) -> Option<(impl Iterator<Item = usize>, impl Iterator<Item = usize>)>;

    fn diff_regular(
        self,
        mov: RegularMove,
        new_position: &Position,
        color: Color,
    ) -> Option<(impl Iterator<Item = usize>, impl Iterator<Item = usize>)>;

    fn redundant(self) -> impl Iterator<Item = impl Iterator<Item = usize>>;
}

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
        Piece::COUNT * NormalizedSquare::COUNT + NUM_CAPTURED_INDEXES - 2
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
            if let Some(captured_piece) = mov.captured {
                if captured_piece != Piece::Wazir {
                    // This is a capture, so we didn't drop the same piece.
                    added.push(Self::captured_feature(
                        captured_piece,
                        new_position.num_captured(captured_piece.with_color(color)) - 1,
                    ));
                }
            }
        } else if let Some(captured_piece) = mov.captured {
            removed = Some(Self::board_feature_unnormalized(captured_piece, mov.to));
        }
        Some((added.into_iter(), removed.into_iter()))
    }

    fn redundant(self) -> impl Iterator<Item = impl Iterator<Item = usize>> {
        iter::once(
            // Wazir positions are redundant because there is always one Wazir.
            NormalizedSquare::all().map(|ns| Self::board_feature(Piece::Wazir, ns)),
        )
    }
}
