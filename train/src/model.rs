use std::{error::Error, path::Path};
use tch::{TchError, Tensor, nn};
use wazir_drop::Features;

/// Input: [batch_size, 2, features.count()]
/// Output: [batch_size]: logit of winning
pub trait EvalModel: nn::Module {
    type Features: Features;

    fn new(features: Self::Features, vs: nn::Path) -> Self;
    fn optimizer(&self, vs: &nn::VarStore, learning_rate: f64) -> Result<nn::Optimizer, TchError>;
    fn project_redundant(&mut self, redundant: &Tensor);
}

pub trait Export {
    fn export(&self, output: &Path, value_scale: f32) -> Result<(), Box<dyn Error>>;
}
