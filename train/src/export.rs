use std::{error::Error, path::PathBuf};

use serde::Deserialize;
use tch::{Device, nn};
use wazir_drop::{Features, PSFeatures};

use crate::{learn::ModelConfig, linear::LinearModel, model::EvalModel, self_play::FeaturesConfig};

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
    match config.features {
        FeaturesConfig::PS => run_with_features(config, PSFeatures),
    }
}

fn run_with_features<F: Features>(config: &Config, features: F) -> Result<(), Box<dyn Error>> {
    match config.model {
        ModelConfig::Linear => run_with_model::<LinearModel<F>>(config, features),
    }
}

pub fn run_with_model<M: EvalModel>(
    config: &Config,
    features: M::Features,
) -> Result<(), Box<dyn Error>> {
    let mut vs = nn::VarStore::new(Device::Cpu);
    let model = M::new(features, vs.root());
    vs.load(&config.weights)?;
    model.export(&config.output, config.value_scale)?;
    Ok(())
}
