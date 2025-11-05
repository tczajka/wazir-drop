use crate::model::EvalModel;
use std::{
    error::Error,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};
use tch::{Tensor, nn};
use wazir_drop::Features;

#[derive(Debug)]
pub struct LinearModel<F: Features> {
    _features: F,
    // [features.count()]
    weights: Tensor,
    // []
    to_move: Tensor,
    // [2]
    side_weights: Tensor,
}

impl<F: Features> EvalModel for LinearModel<F> {
    type Features = F;

    fn new(features: F, vs: nn::Path) -> Self {
        let weights =
            (&vs / "weights").var("weight", &[features.count() as i64], nn::Init::Const(0.0));
        let to_move = (&vs / "to_move").var("weight", &[], nn::Init::Const(0.0));
        let side_weights = Tensor::from_slice(&[1.0f32, -1.0f32]).to_device(vs.device());

        Self {
            _features: features,
            weights,
            to_move,
            side_weights,
        }
    }

    fn project_redundant(&mut self, redundant: &Tensor) {
        let x = self.weights.dot(redundant) / redundant.sum(None);
        self.weights -= &x * redundant;
        self.to_move += &x;
    }

    fn export(&self, output: &Path, value_scale: f32) -> Result<(), Box<dyn Error>> {
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
        write!(f, "pub static FEATURES: [i16; {}] = [", weights.len())?;
        for (i, &weight) in weights.iter().enumerate() {
            if i.is_multiple_of(10) {
                write!(f, "\n    ")?;
            } else {
                write!(f, " ")?;
            }
            write!(f, "{weight},")?;
        }
        writeln!(f, "\n];")?;
        Ok(())
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
