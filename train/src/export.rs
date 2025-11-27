use extra::PSFeatures;
use serde::Deserialize;
use std::{error::Error, path::PathBuf};
use tch::{Device, nn};
use wazir_drop::{Features, WPSFeatures};

use crate::{
    config::FeaturesConfig,
    linear::{self, LinearModel},
    model::{EvalModel, Export},
    nnue::{self, NnueModel},
};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub weights: PathBuf,
    pub output: PathBuf,
    pub features: FeaturesConfig,
    pub model: ModelConfig,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelConfig {
    Linear { export: linear::ExportConfig },
    Nnue { config: nnue::Config },
}

pub fn run(config: &Config) -> Result<(), Box<dyn Error>> {
    match config.features {
        FeaturesConfig::PS => run_with_features(PSFeatures, config),
        FeaturesConfig::WPS => run_with_features(WPSFeatures, config),
    }
}

pub fn run_with_features<F: Features>(features: F, config: &Config) -> Result<(), Box<dyn Error>>
where
    LinearModel<F>: Export<ExportConfig = linear::ExportConfig>,
{
    match &config.model {
        ModelConfig::Linear { export } => {
            run_with_model::<LinearModel<F>>(features, config, &(), export)
        }
        ModelConfig::Nnue {
            config: model_config,
        } => run_with_model::<NnueModel<F>>(features, config, model_config, &()),
    }
}

pub fn run_with_model<M: EvalModel + Export>(
    features: M::Features,
    config: &Config,
    model_config: &M::Config,
    export_config: &M::ExportConfig,
) -> Result<(), Box<dyn Error>> {
    let mut vs = nn::VarStore::new(Device::Cpu);
    let model = M::new(features, vs.root(), model_config);
    vs.load(&config.weights)?;
    model.export(&config.output, export_config)?;
    log::info!("Exported model to {}", config.output.display());
    Ok(())
}
