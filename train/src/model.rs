use std::{error::Error, path::Path};
use tch::{TchError, nn};
use wazir_drop::Features;

/// Input: [batch_size, 2, features.count()]
/// Output: [batch_size]: logit of winning
pub trait EvalModel: nn::Module {
    type Features: Features;
    type Config;

    fn new(features: Self::Features, vs: nn::Path, config: &Self::Config) -> Self;
    fn optimizer(&self, vs: &nn::VarStore) -> Result<nn::Optimizer, TchError>;
    fn clean_up(&mut self);
}

pub trait Export {
    fn export(&self, output: &Path, value_scale: f32) -> Result<(), Box<dyn Error>>;
}
