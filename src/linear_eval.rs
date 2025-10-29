use crate::{
    enums::EnumMap, linear_eval_piece_square_weights, Color, Evaluator, Features,
    PieceSquareFeatures,
};

#[derive(Debug)]
pub struct LinearEvaluator<F> {
    features: F,
    to_move_weight: i16,
    feature_weights: Vec<i16>,
}

impl<F: Features> LinearEvaluator<F> {
    pub fn new(features: F, to_move_weight: i16, feature_weights: &[i16]) -> Self {
        assert_eq!(feature_weights.len(), features.count());
        Self {
            features,
            to_move_weight,
            feature_weights: feature_weights.to_vec(),
        }
    }
}

impl Default for LinearEvaluator<PieceSquareFeatures> {
    fn default() -> Self {
        Self::new(
            PieceSquareFeatures,
            linear_eval_piece_square_weights::TO_MOVE,
            &linear_eval_piece_square_weights::FEATURES,
        )
    }
}

impl<F: Features> Evaluator for LinearEvaluator<F> {
    type Accumulator = i32;
    type Features = F;

    fn features(&self) -> &Self::Features {
        &self.features
    }

    fn new_accumulator(&self) -> Self::Accumulator {
        0
    }

    fn add_feature(&self, accumulator: &mut Self::Accumulator, feature: usize) {
        *accumulator += i32::from(self.feature_weights[feature]);
    }

    fn remove_feature(&self, accumulator: &mut Self::Accumulator, feature: usize) {
        *accumulator -= i32::from(self.feature_weights[feature]);
    }

    fn evaluate(&self, accumulators: &EnumMap<Color, Self::Accumulator>, to_move: Color) -> i32 {
        accumulators[to_move] - accumulators[to_move.opposite()] + i32::from(self.to_move_weight)
    }
}
