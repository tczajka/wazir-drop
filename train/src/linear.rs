use crate::model::EvalModel;
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
