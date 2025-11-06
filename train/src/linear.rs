use crate::model::{EvalModel, Export};
use serde::Deserialize;
use std::{
    error::Error,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};
use tch::{
    TchError, Tensor,
    nn::{self, OptimizerConfig},
};
use wazir_drop::{
    Coord, Features, NormalizedSquare, PSFeatures, Piece, Square, WPSFeatures, enums::SimpleEnumExt,
};

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    learning_rate: f64,
    weight_decay: f64,
}

#[derive(Debug)]
pub struct LinearModel<F: Features> {
    _features: F,
    config: Config,
    // [features.count()]
    weights: Tensor,
    // []
    to_move: Tensor,
    // Always [1.0, -1.0]
    side_weights: Tensor,
}

impl<F: Features> EvalModel for LinearModel<F> {
    type Features = F;
    type Config = Config;

    fn new(features: F, vs: nn::Path, config: &Config) -> Self {
        let weights =
            (&vs / "weights").var("weight", &[features.count() as i64], nn::Init::Const(0.0));
        let to_move = (&vs / "to_move").var("weight", &[], nn::Init::Const(0.0));
        let side_weights = Tensor::from_slice(&[1.0f32, -1.0f32]).to_device(vs.device());

        Self {
            _features: features,
            config: config.clone(),
            weights,
            to_move,
            side_weights,
        }
    }

    fn optimizer(&self, vs: &nn::VarStore) -> Result<nn::Optimizer, TchError> {
        let adamw = nn::AdamW {
            wd: self.config.weight_decay,
            ..Default::default()
        };
        adamw.build(vs, self.config.learning_rate)
    }
}

impl<F: Features> nn::Module for LinearModel<F> {
    fn forward(&self, xs: &Tensor) -> Tensor {
        // xs: [batch_size, 2, features.count()]
        let weights = self
            .weights
            .unsqueeze(1)
            .expand([xs.size()[0], -1, -1], false);
        // weights: [batch_size, features.count(), 1]
        let res = xs.bmm(&weights).squeeze_dim(2);
        // res: [batch_size, 2]
        res.matmul(&self.side_weights) + &self.to_move
    }
}

impl Export for LinearModel<PSFeatures> {
    fn export(&self, output: &Path, value_scale: f32) -> Result<(), Box<dyn Error>> {
        let _guard = tch::no_grad_guard();
        let max_abs = self.weights.max().max_other(&self.to_move.abs().max());
        let max_abs = f32::try_from(max_abs).unwrap();
        println!("max |weight| = {max_abs:.6}");
        let mut f = BufWriter::new(File::create(output)?);
        let to_move = (value_scale * &self.to_move).round();
        let to_move: i16 = to_move.try_into().expect("out of range");
        writeln!(f, "pub static TO_MOVE: i16 = {to_move};")?;
        writeln!(f)?;
        let weights = (value_scale * &self.weights).round();
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
    fn export(&self, output: &Path, value_scale: f32) -> Result<(), Box<dyn Error>> {
        let _guard = tch::no_grad_guard();
        let max_abs = self.weights.max().max_other(&self.to_move.abs().max());
        let max_abs = f32::try_from(max_abs).unwrap();
        println!("max |weight| = {max_abs:.6}");
        let mut f = BufWriter::new(File::create(output)?);
        let to_move = (value_scale * &self.to_move).round();
        let to_move: i16 = to_move.try_into().expect("out of range");
        writeln!(f, "pub static TO_MOVE: i16 = {to_move};")?;
        writeln!(f)?;
        let weights = (value_scale * &self.weights).round();
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
                            write!(f, " {},", weights[next])?;
                            next += 1;
                        }
                        writeln!(f)?;
                    }
                    for square in NormalizedSquare::all() {
                        assert_eq!(next, PSFeatures::board_feature(piece, square));
                        next += 1;
                    }
                    writeln!(f)?;
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
