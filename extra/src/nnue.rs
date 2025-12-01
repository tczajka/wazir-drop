use crate::{
    nnue_wps_weights::{self, EMBEDDING_SIZE},
    vector::Vector16,
};
use wazir_drop::{Color, Evaluator, WPSFeatures, constants::Eval, enums::EnumMap};

const fn exact_div(a: usize, b: usize) -> usize {
    if a % b != 0 {
        panic!("exact_div");
    }
    a / b
}

type EmbeddingVector = Vector16<{ exact_div(EMBEDDING_SIZE, 8) }>;

pub struct Nnue {
    embedding_weights: Vec<EmbeddingVector>,
    embedding_bias: EmbeddingVector,
}

impl Nnue {
    pub fn new() -> Self {
        todo!()
    }
}

impl Default for Nnue {
    fn default() -> Self {
        Self::new()
    }
}

impl Evaluator for Nnue {
    type Accumulator = EmbeddingVector;
    type Features = WPSFeatures;

    fn features(&self) -> Self::Features {
        WPSFeatures
    }

    fn new_accumulator(&self) -> Self::Accumulator {
        self.embedding_bias
    }

    fn add_feature(&self, accumulator: &mut Self::Accumulator, feature: usize) {
        *accumulator += &self.embedding_weights[feature];
    }

    fn remove_feature(&self, accumulator: &mut Self::Accumulator, feature: usize) {
        *accumulator -= &self.embedding_weights[feature];
    }

    fn evaluate(&self, accumulators: &EnumMap<Color, Self::Accumulator>, to_move: Color) -> Eval {
        todo!()
    }

    fn scale(&self) -> f32 {
        nnue_wps_weights::SCALE
    }
}
