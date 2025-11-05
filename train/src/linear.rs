use crate::{
    model::{EvalModel, Export},
    util::sparse_1d_tensor,
};
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
use wazir_drop::{Features, NormalizedSquare, PSFeatures, Piece, enums::SimpleEnumExt};

#[derive(Debug, Deserialize)]
pub struct Config {
    learning_rate: f64,
}

#[derive(Debug)]
pub struct LinearModel<F: Features> {
    _features: F,
    learning_rate: f64,
    // [features.count()]
    weights: Tensor,
    // []
    to_move: Tensor,
    // Always [1.0, -1.0]
    side_weights: Tensor,
    // Redundant projection.
    // [D, features.count()]
    redundant: Tensor,
    // [features.count(), D]
    // redundant * redundant_inv = I
    redundant_inv: Tensor,
}

impl<F: Features> EvalModel for LinearModel<F> {
    type Features = F;
    type Config = Config;

    fn new(features: F, vs: nn::Path, config: &Config) -> Self {
        let weights =
            (&vs / "weights").var("weight", &[features.count() as i64], nn::Init::Const(0.0));
        let to_move = (&vs / "to_move").var("weight", &[], nn::Init::Const(0.0));
        let side_weights = Tensor::from_slice(&[1.0f32, -1.0f32]).to_device(vs.device());

        let redundant: Vec<Tensor> = features
            .redundant()
            .map(|r| sparse_1d_tensor(r, features.count()))
            .collect();
        let redundant = Tensor::stack(&redundant, 0).to_device(vs.device());
        // rows of redundant are orthogonal
        // R @ R^T = diag(R^2.sum(1))
        // R @ R^T @ diag(R^2.sum(1))^-1 = I
        // R_inv = R^T @ diag(R^2.sum(1))^-1
        // mult: [D]
        let mult = redundant.square().sum_dim_intlist(1, true, None);
        let redundant_inv = redundant.transpose(0, 1) / mult;

        Self {
            _features: features,
            learning_rate: config.learning_rate,
            weights,
            to_move,
            side_weights,
            redundant,
            redundant_inv,
        }
    }

    fn optimizer(&self, vs: &nn::VarStore) -> Result<nn::Optimizer, TchError> {
        nn::Adam::default().build(vs, self.learning_rate)
    }

    fn clean_up(&mut self) {
        let _guard = tch::no_grad_guard();
        // W -= R^(-1) * R * W
        self.weights -= self
            .redundant_inv
            .mm(&self.redundant.mm(&self.weights.unsqueeze(1)))
            .squeeze_dim(1);
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
