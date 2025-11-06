use serde::Deserialize;
use std::{error::Error, path::PathBuf};
use tch::{Device, nn};
use wazir_drop::{PSFeatures, WPSFeatures};

use crate::{
    learn::ModelConfig,
    linear::LinearModel,
    model::{EvalModel, Export},
    self_play::FeaturesConfig,
};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub weights: PathBuf,
    pub output: PathBuf,
    pub features: FeaturesConfig,
    pub model: ModelConfig,
    pub value_scale: f32,
}

pub fn run(config: &Config) -> Result<(), Box<dyn Error>> {
    match (&config.features, &config.model) {
        (FeaturesConfig::PS, ModelConfig::Linear(c)) => {
            run_with_model::<LinearModel<_>>(PSFeatures, config, c)
        }
        (FeaturesConfig::WPS, ModelConfig::Linear(c)) => {
            run_with_model::<LinearModel<_>>(WPSFeatures, config, c)
        }
    }
}

pub fn run_with_model<M: EvalModel + Export>(
    features: M::Features,
    config: &Config,
    model_config: &M::Config,
) -> Result<(), Box<dyn Error>> {
    let mut vs = nn::VarStore::new(Device::Cpu);
    let model = M::new(features, vs.root(), model_config);
    vs.load(&config.weights)?;
    model.export(&config.output, config.value_scale)?;
    Ok(())
}
