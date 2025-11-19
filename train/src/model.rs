use std::{error::Error, path::Path};
use tch::{Tensor, nn};
use wazir_drop::Features;

pub trait EvalModel {
    type Features: Features;
    type Config;

    fn new(features: Self::Features, vs: nn::Path, config: &Self::Config) -> Self;

    /// features: [num features in a batch]
    /// offsets: [batch_size, 2] -> indices into features
    /// output: [batch_size] -> logit of winning
    fn forward(&mut self, features: &Tensor, offsets: &Tensor) -> Tensor;

    fn fixup(&mut self);

    fn num_layers(&self) -> usize;

    /// layer < num_layers
    /// [num_weights]
    fn layer_weights(&self, layer: usize) -> Tensor;

    /// layer < num_layers - 1
    /// [batch_size, layer_size]
    fn activations(&self, layer: usize) -> Tensor;
}

pub trait Export {
    fn export(&self, output: &Path, value_scale: f32) -> Result<(), Box<dyn Error>>;
}
