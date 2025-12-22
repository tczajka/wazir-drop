use crate::{
    base128::Base128Decoder,
    constants::Eval,
    enums::EnumMap,
    nnue_weights::{EMBEDDING_SIZE, HIDDEN_SIZES, HIDDEN_WEIGHT_BITS, SCALE, WEIGHTS},
    vector::{crelu16, crelu32, dot_product, mul_add, vector_concat, Vector16, Vector32, Vector8},
    Color, Evaluator, Features, WPSFeatures,
};
use std::array;

const fn exact_div(a: usize, b: usize) -> usize {
    if a % b != 0 {
        panic!("exact_div");
    }
    a / b
}

type EmbeddingVector = Vector16<{ exact_div(EMBEDDING_SIZE, 8) }>;

pub struct Nnue {
    features: WPSFeatures,
    embedding_weights: Vec<EmbeddingVector>,
    embedding_bias: EmbeddingVector,
    hidden_0_weights: [Vector8<{ 2 * exact_div(EMBEDDING_SIZE, 16) }>; HIDDEN_SIZES[0]],
    hidden_0_bias: Vector32<{ exact_div(HIDDEN_SIZES[0], 4) }>,
    final_layer_weights: Vector8<{ exact_div(HIDDEN_SIZES[0], 16) }>,
    final_layer_bias: i32,
}

impl Nnue {
    pub fn new() -> Self {
        let features = WPSFeatures;
        let mut decoder = Base128Decoder::new(WEIGHTS);
        let embedding_weights = (0..features.count())
            .map(|_| {
                Self::decode_vector16::<EMBEDDING_SIZE, { exact_div(EMBEDDING_SIZE, 8) }>(
                    &mut decoder,
                )
            })
            .collect();
        let embedding_bias =
            Self::decode_vector16::<EMBEDDING_SIZE, { exact_div(EMBEDDING_SIZE, 8) }>(&mut decoder);
        let hidden_0_weights = array::from_fn(|_| {
            Self::decode_vector8::<{ 2 * EMBEDDING_SIZE }, { exact_div(2 * EMBEDDING_SIZE, 16) }>(
                &mut decoder,
            )
        });
        let hidden_0_bias = Self::decode_vector32::<
            { HIDDEN_SIZES[0] },
            { exact_div(HIDDEN_SIZES[0], 4) },
        >(&mut decoder);
        let final_layer_weights = Self::decode_vector8::<
            { HIDDEN_SIZES[0] },
            { exact_div(HIDDEN_SIZES[0], 16) },
        >(&mut decoder);
        let final_layer_bias = decoder.decode_varint();

        decoder.finish();

        Self {
            features,
            embedding_weights,
            embedding_bias,
            hidden_0_weights,
            hidden_0_bias,
            final_layer_weights,
            final_layer_bias,
        }
    }

    fn decode_vector8<const N: usize, const N16: usize>(
        decoder: &mut Base128Decoder,
    ) -> Vector8<N16> {
        assert_eq!(N, 16 * N16);
        let arr: [i8; N] = array::from_fn(|_| decoder.decode_varint().try_into().unwrap());
        (&arr).into()
    }

    fn decode_vector16<const N: usize, const N8: usize>(
        decoder: &mut Base128Decoder,
    ) -> Vector16<N8> {
        assert_eq!(N, 8 * N8);
        let arr: [i16; N] = array::from_fn(|_| decoder.decode_varint().try_into().unwrap());
        (&arr).into()
    }

    fn decode_vector32<const N: usize, const N4: usize>(
        decoder: &mut Base128Decoder,
    ) -> Vector32<N4> {
        assert_eq!(N, 4 * N4);
        let arr: [i32; N] = array::from_fn(|_| decoder.decode_varint());
        (&arr).into()
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
        self.features
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
        let x: EnumMap<Color, Vector8<{ exact_div(EMBEDDING_SIZE, 16) }>> =
            EnumMap::from_fn(|color| crelu16(&accumulators[color]));
        let x = vector_concat(&x[to_move], &x[to_move.opposite()]);
        assert_eq!(HIDDEN_SIZES.len(), 1);
        let x = mul_add::<
            { HIDDEN_SIZES[0] },
            { exact_div(HIDDEN_SIZES[0], 4) },
            { 2 * exact_div(EMBEDDING_SIZE, 16) },
            { HIDDEN_WEIGHT_BITS[0] },
        >(&self.hidden_0_weights, &x, &self.hidden_0_bias);
        let x = crelu32(&x);
        dot_product(&self.final_layer_weights, &x, self.final_layer_bias)
    }

    fn scale(&self) -> f64 {
        SCALE
    }
}
