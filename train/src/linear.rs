use tch::{Tensor, nn};
use wazir_drop::Features;

#[derive(Debug)]
pub struct LinearModel<F: Features> {
    features: F,
    layer0: nn::Linear,
}

impl<F: Features> LinearModel<F> {
    pub fn new(features: F, vs: nn::Path) -> Self {
        let layer0 = nn::linear(
            vs / "layer0",
            features.count().try_into().unwrap(),
            1,
            nn::LinearConfig::default(),
        );

        Self { features, layer0 }
    }
}

impl<F: Features> nn::Module for LinearModel<F> {
    fn forward(&self, xs: &Tensor) -> Tensor {
        self.layer0.forward(xs)
    }
}
