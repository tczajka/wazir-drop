use crate::model::{EvalModel, Export};
use extra::PSFeatures;
use serde::Deserialize;
use std::{
    error::Error,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};
use tch::{
    IndexOp, Tensor,
    nn,
};
use wazir_drop::{
    Coord, Features, NormalizedSquare, Piece, Square, WPSFeatures, enums::SimpleEnumExt,
};

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LearnConfig {
    max_weight: f64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExportConfig {
    value_scale: f32,
}

#[derive(Debug)]
pub struct LinearModel<F: Features> {
    _features: F,
    // [features.count()]
    weights: Tensor,
    // []
    to_move: Tensor,
}

impl<F: Features> EvalModel for LinearModel<F> {
    type Features = F;
    type Config = ();
    type LearnConfig = LearnConfig;

    fn new(features: F, vs: nn::Path, _config: &()) -> Self {
        let weights = vs.var("weights", &[features.count() as i64], nn::Init::Const(0.0));
        let to_move = vs.var("to_move", &[], nn::Init::Const(0.0));

        Self {
            _features: features,
            weights,
            to_move,
        }
    }

    fn forward(&mut self, features: &Tensor, offsets: &Tensor) -> Tensor {
        let (embedding, _, _, _) = Tensor::embedding_bag::<&Tensor>(
            &self.weights.unsqueeze(1),
            features,
            &offsets.reshape([-1]),
            false, /* scale_grad_by_freq */
            0,     /* mode = sum */
            false, /* sparse */
            None,  /* per_sample_weights */
            false, /* include_last_offset */
        );
        // embedding: [batch_size * 2, 1]
        let embedding = embedding.reshape([-1, 2]);
        // embedding: [batch_size, 2]
        embedding.i((.., 0)) - embedding.i((.., 1)) + &self.to_move
    }

    fn fixup(&mut self, learn_config: &Self::LearnConfig) {
        let _guard = tch::no_grad_guard();
        _ = self
            .weights
            .clamp_(-learn_config.max_weight, learn_config.max_weight);
    }

    fn num_layers(&self) -> usize {
        1
    }

    fn layer_weights(&self, layer: usize) -> Tensor {
        assert_eq!(layer, 0);
        self.weights.shallow_clone()
    }

    fn activations(&self, _layer: usize) -> Tensor {
        panic!("no activations in linear model");
    }
}

impl Export for LinearModel<PSFeatures> {
    type ExportConfig = ExportConfig;

    fn export(&self, output: &Path, export_config: &ExportConfig) -> Result<(), Box<dyn Error>> {
        let _guard = tch::no_grad_guard();
        let max_abs = self.weights.abs().max().max_other(&self.to_move.abs());
        let max_abs = f32::try_from(max_abs).unwrap();
        println!("max |weight| = {max_abs:.6}");
        let mut f = BufWriter::new(File::create(output)?);
        let to_move = (export_config.value_scale * &self.to_move).round();
        let to_move: i16 = to_move.try_into().expect("out of range");
        writeln!(f, "pub static TO_MOVE: i16 = {to_move};")?;
        writeln!(f)?;
        let weights = (export_config.value_scale * &self.weights).round();
        let weights: Vec<i16> = weights.try_into().expect("out of range");
        writeln!(f, "#[rustfmt::skip]")?;
        writeln!(f, "pub static FEATURES: [i16; {}] = [", weights.len())?;
        let mut next = 0;
        for piece in Piece::all() {
            writeln!(f, "    // {}", piece.long_name())?;
            write!(f, "   ")?;
            for square in NormalizedSquare::all() {
                assert_eq!(next, PSFeatures::board_feature(piece, square));
                write!(f, " {},", weights[next])?;
                next += 1;
            }
            writeln!(f)?;
        }
        for piece in Piece::all_non_wazir() {
            writeln!(f, "    // captured {}", piece.long_name())?;
            write!(f, "   ")?;
            for index in 0..piece.total_count() {
                assert_eq!(next, PSFeatures::captured_feature(piece, index));
                write!(f, " {},", weights[next])?;
                next += 1;
            }
            writeln!(f)?;
        }
        writeln!(f, "];")?;
        assert_eq!(next, weights.len());
        Ok(())
    }
}

impl Export for LinearModel<WPSFeatures> {
    type ExportConfig = ExportConfig;

    fn export(&self, output: &Path, export_config: &ExportConfig) -> Result<(), Box<dyn Error>> {
        let _guard = tch::no_grad_guard();
        let max_abs = self.weights.max().max_other(&self.to_move.abs().max());
        let max_abs = f32::try_from(max_abs).unwrap();
        println!("max |weight| = {max_abs:.6}");
        let mut f = BufWriter::new(File::create(output)?);
        let to_move = (export_config.value_scale * &self.to_move).round();
        let to_move: i16 = to_move.try_into().expect("out of range");
        writeln!(f, "pub static TO_MOVE: i16 = {to_move};")?;
        writeln!(f)?;
        let weights = (export_config.value_scale * &self.weights).round();
        let weights: Vec<i16> = weights.try_into().expect("out of range");
        writeln!(f, "#[rustfmt::skip]")?;
        writeln!(f, "pub static FEATURES: [i16; {}] = [", weights.len())?;
        let mut next = 0;
        for wazir_nsquare in NormalizedSquare::all() {
            writeln!(f, "// wazir: {wazir_nsquare}")?;
            for is_other_color in [false, true] {
                let color_name = if is_other_color { "other" } else { "same" };
                for piece in Piece::all() {
                    if (piece, is_other_color) == (Piece::Wazir, false) {
                        continue;
                    }
                    writeln!(f, "    // {color_name} {p}", p = piece.long_name())?;
                    for y in 0..Coord::HEIGHT {
                        write!(f, "   ")?;
                        for x in 0..Coord::WIDTH {
                            let square = Square::from(Coord::new(x, y));
                            assert_eq!(
                                next,
                                WPSFeatures::board_feature(
                                    wazir_nsquare,
                                    is_other_color,
                                    piece,
                                    square
                                )
                            );
                            write!(f, "{:5},", weights[next])?;
                            next += 1;
                        }
                        writeln!(f)?;
                    }
                }
            }
            for is_other_color in [false, true] {
                let color_name = if is_other_color { "other" } else { "same" };
                for piece in Piece::all_non_wazir() {
                    writeln!(f, "    // captured {color_name} {p}", p = piece.long_name())?;
                    write!(f, "   ")?;
                    for index in 0..piece.total_count() {
                        assert_eq!(
                            next,
                            WPSFeatures::captured_feature(
                                wazir_nsquare,
                                is_other_color,
                                piece,
                                index
                            )
                        );
                        write!(f, " {},", weights[next])?;
                        next += 1;
                    }
                    writeln!(f)?;
                }
            }
        }
        writeln!(f, "];")?;
        assert_eq!(next, weights.len());
        Ok(())
    }
}
