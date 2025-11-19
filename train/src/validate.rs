use std::{error::Error, path::PathBuf, time::Instant};
use extra::PSFeatures;
use serde::Deserialize;
use tch::{Device, Reduction, Tensor, nn};
use wazir_drop::{Features, WPSFeatures};
use crate::{config::{FeaturesConfig, ModelConfig}, data::{DatasetConfig, DatasetIterator}, linear::LinearModel, model::EvalModel, nnue::NnueModel};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    dataset: DatasetConfig,
    weights: PathBuf,
    model: ModelConfig,
}

pub fn run(config: &Config) -> Result<(), Box<dyn Error>> {
    match config.dataset.features {
        FeaturesConfig::PS => run_with_features(PSFeatures, config),
        FeaturesConfig::WPS => run_with_features(WPSFeatures, config),
    }
}

fn run_with_features<F: Features>(features: F, config: &Config) -> Result<(), Box<dyn Error>> {
    match &config.model {
        ModelConfig::Linear(c) => run_with_model::<LinearModel<F>>(features, config, c),
        ModelConfig::Nnue(c) => run_with_model::<NnueModel<F>>(features, config, c),
    }
}

fn run_with_model<M: EvalModel>(
    features: M::Features,
    config: &Config,
    model_config: &M::Config,
) -> Result<(), Box<dyn Error>> {
    let device = Device::cuda_if_available();
    log::info!("Validating using device: {device:?}");
    let mut vs = nn::VarStore::new(device);
    let mut model = M::new(features, vs.root(), model_config);
    vs.load(&config.weights)?;

    let mut num_samples = 0;
    let mut total_loss: f64 = 0.0;
    let start_time = Instant::now();
    let mut dataset_iterator = DatasetIterator::new(&config.dataset)?;
    while let Some(batch) = dataset_iterator.next_batch()? {
        let batch = batch.to_device(device);
        let values = model.forward(&batch.features, &batch.offsets);
        let loss = values.binary_cross_entropy_with_logits::<Tensor>(
            &batch.outputs,
            None,
            None,
            Reduction::Mean,
        );
        num_samples += batch.size;
        total_loss += batch.size as f64 * f64::try_from(&loss).unwrap();
    }
    let elapsed_time = start_time.elapsed().as_secs_f64();
    log::info!(
        "samples={num_samples} time={elapsed_time:.2}s \
        samples/s={samples_per_second:.0} loss={loss:.6}",
        samples_per_second = num_samples as f64 / elapsed_time,
        loss = total_loss / num_samples as f64,
    );
    Ok(())
}