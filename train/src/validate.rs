use std::{error::Error, path::PathBuf};
use extra::PSFeatures;
use serde::Deserialize;
use wazir_drop::{Features, WPSFeatures};
use crate::{config::{FeaturesConfig, ModelConfig}, nnue};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    validation_data: PathBuf,
    weights: Option<PathBuf>,
    input_value_scale: f32,
    features: FeaturesConfig,
    model: ModelConfig,
    chunk_size: usize,
    batch_size: usize,
    outcome_weight: f32,
    log_period_seconds: f32,
}


pub fn run(config: &Config) -> Result<(), Box<dyn Error>> {
    match config.features {
        FeaturesConfig::PS => run_with_features(PSFeatures, config),
        FeaturesConfig::WPS => run_with_features(WPSFeatures, config),
    }
}

fn run_with_features<F: Features>(features: F, config: &Config) -> Result<(), Box<dyn Error>> {
    match &config.model {
        ModelConfig::Nnue(c) => run_with_nnue(features, config, c),
        ModelConfig::Linear(c) => panic!("validation with linear model is not implemented"),
    }
}

fn run_with_nnue<F: Features>(
    features: F,
    config: &Config,
    model_config: &nnue::Config,
) -> Result<(), Box<dyn Error>> {
    // TODO
    Ok(())
}