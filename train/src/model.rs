use std::{error::Error, path::Path};
use tch::{TchError, Tensor, nn};
use wazir_drop::Features;

pub trait EvalModel {
    type Features: Features;
    type Config;

    fn new(features: Self::Features, vs: nn::Path, config: &Self::Config) -> Self;

    /// features: [num features in a batch]
    /// offsets: [batch_size, 2] -> indices into features
    /// output: [batch_size] -> logit of winning
    fn forward(&self, features: &Tensor, offsets: &Tensor) -> Tensor;

    fn optimizer(&self, vs: &nn::VarStore) -> Result<nn::Optimizer, TchError>;

    fn fixup(&mut self);
}

pub trait Export {
    fn export(&self, output: &Path, value_scale: f32) -> Result<(), Box<dyn Error>>;
}
