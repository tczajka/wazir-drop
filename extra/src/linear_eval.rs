use crate::{linear_ps_weights, linear_wps_weights, ps_features::PSFeatures};
use wazir_drop::{Color, Evaluator, Features, WPSFeatures, constants::Eval, enums::EnumMap};

#[derive(Debug)]
pub struct LinearEvaluator<F> {
    features: F,
    to_move_weight: i16,
    feature_weights: Vec<i16>,
    scale: f32,
}

impl<F: Features> LinearEvaluator<F> {
    pub fn new(features: F, to_move_weight: i16, feature_weights: &[i16], scale: f32) -> Self {
        assert_eq!(feature_weights.len(), features.count());
        Self {
            features,
            to_move_weight,
            feature_weights: feature_weights.to_vec(),
            scale,
        }
    }
}

impl<F: Features> Evaluator for LinearEvaluator<F> {
    type Accumulator = Eval;
    type Features = F;

    fn features(&self) -> Self::Features {
        self.features
    }

    fn new_accumulator(&self) -> Self::Accumulator {
        0
    }

    fn add_feature(&self, accumulator: &mut Self::Accumulator, feature: usize) {
        *accumulator += Eval::from(self.feature_weights[feature]);
    }

    fn remove_feature(&self, accumulator: &mut Self::Accumulator, feature: usize) {
        *accumulator -= Eval::from(self.feature_weights[feature]);
    }

    fn evaluate(&self, accumulators: &EnumMap<Color, Self::Accumulator>, to_move: Color) -> Eval {
        accumulators[to_move] - accumulators[to_move.opposite()] + Eval::from(self.to_move_weight)
    }

    fn scale(&self) -> f32 {
        self.scale
    }
}

impl Default for LinearEvaluator<PSFeatures> {
    fn default() -> Self {
        Self::new(
            PSFeatures,
            linear_ps_weights::TO_MOVE,
            &linear_ps_weights::FEATURES,
            linear_ps_weights::SCALE,
        )
    }
}

impl Default for LinearEvaluator<WPSFeatures> {
    fn default() -> Self {
        Self::new(
            WPSFeatures,
            linear_wps_weights::TO_MOVE,
            &linear_wps_weights::FEATURES,
            linear_wps_weights::SCALE,
        )
    }
}
