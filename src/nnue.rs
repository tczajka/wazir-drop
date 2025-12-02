use crate::{
    base128_decoder::Base128Decoder,
    constants::Eval,
    enums::EnumMap,
    nnue_weights::{EMBEDDING_SIZE, HIDDEN_SIZES, SCALE, WEIGHTS},
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
    /*
    hidden_0_weights: [Vector8<{ 2 * exact_div(EMBEDDING_SIZE, 16) }>; HIDDEN_SIZES[0]],
    hidden_0_bias: Vector32<{ exact_div(HIDDEN_SIZES[0], 4) }>,
    hidden_1_weights: [Vector8<{ exact_div(HIDDEN_SIZES[0], 16) }>; HIDDEN_SIZES[1]],
    hidden_1_bias: Vector32<{ exact_div(HIDDEN_SIZES[1], 16) }>,
    */
    final_layer_weights: Vector8<{ 2 * exact_div(EMBEDDING_SIZE, 16) }>,
    final_layer_bias: i32,
}

impl Nnue {
    pub fn new() -> Self {
        let features = WPSFeatures;
        let mut decoder = Base128Decoder::new(WEIGHTS);
        let embedding_weights = (0..features.count())
            .map(|_| Self::decode_vector16::<EMBEDDING_SIZE, _>(&mut decoder))
            .collect();
        let embedding_bias = Self::decode_vector16::<EMBEDDING_SIZE, _>(&mut decoder);
        /*
        let hidden_0_weights =
            array::from_fn(|_| Self::decode_vector8::<{ 2 * EMBEDDING_SIZE }, _>(&mut decoder));
        let hidden_0_bias = Self::decode_vector32::<{ HIDDEN_SIZES[0] }, _>(&mut decoder);
        let hidden_1_weights =
            array::from_fn(|_| Self::decode_vector8::<{ HIDDEN_SIZES[0] }, _>(&mut decoder));
        let hidden_1_bias = Self::decode_vector32::<{ HIDDEN_SIZES[1] }, _>(&mut decoder);
        */
        let final_layer_weights = Self::decode_vector8::<{ 2 * EMBEDDING_SIZE }, _>(&mut decoder);
        let final_layer_bias = decoder.decode_varint();

        decoder.finish();

        Self {
            features,
            embedding_weights,
            embedding_bias,
            /*
            hidden_0_weights,
            hidden_0_bias,
            hidden_1_weights,
            hidden_1_bias,
            */
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
        let a: Vector8<{ exact_div(EMBEDDING_SIZE, 16) }> = crelu16(&accumulators[to_move]);
        let b: Vector8<{ exact_div(EMBEDDING_SIZE, 16) }> =
            crelu16(&accumulators[to_move.opposite()]);
        let x = vector_concat(&a, &b);
        assert_eq!(HIDDEN_SIZES.len(), 0);
        dot_product(&self.final_layer_weights, &x, self.final_layer_bias)
    }

    fn scale(&self) -> f32 {
        SCALE
    }
}
