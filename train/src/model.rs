use std::{error::Error, path::Path};

use tch::{Tensor, nn};
use wazir_drop::Features;

/// Input: [batch_size, 2, features.count()]
/// Output: [batch_size]: logit of winning
pub trait EvalModel: nn::Module {
    type Features: Features;

    fn new(features: Self::Features, vs: nn::Path) -> Self;
    fn project_redundant(&mut self, redundant: &Tensor);
    fn export(&self, output: &Path, value_scale: f32) -> Result<(), Box<dyn Error>>;
}
