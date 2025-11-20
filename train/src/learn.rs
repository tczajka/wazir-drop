use crate::{
    config::FeaturesConfig,
    data::{DatasetConfig, DatasetIterator},
    linear::{self, LinearModel},
    model::EvalModel,
    nnue::{self, NnueModel},
};
use extra::PSFeatures;
use serde::Deserialize;
use std::{error::Error, path::PathBuf, time::Instant};
use tch::{
    Device, Reduction, Tensor,
    nn::{self, OptimizerConfig},
};
use wazir_drop::{Features, WPSFeatures};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    dataset: DatasetConfig,
    load_weights: Option<PathBuf>,
    save_weights: PathBuf,
    model: ModelConfig,
    learning_rate: f64,
    epochs: u32,
    log_period_seconds: f32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelConfig {
    Linear {
        learn: linear::LearnConfig,
    },
    Nnue {
        config: nnue::Config,
        learn: nnue::LearnConfig,
    },
}

pub fn run(config: &Config) -> Result<(), Box<dyn Error>> {
    match config.dataset.features {
        FeaturesConfig::PS => run_with_features(PSFeatures, config),
        FeaturesConfig::WPS => run_with_features(WPSFeatures, config),
    }
}

fn run_with_features<F: Features>(features: F, config: &Config) -> Result<(), Box<dyn Error>> {
    match &config.model {
        ModelConfig::Linear { learn } => {
            run_with_model::<LinearModel<F>>(features, config, &(), learn)
        }
        ModelConfig::Nnue {
            config: nnue_config,
            learn,
        } => run_with_model::<NnueModel<F>>(features, config, nnue_config, learn),
    }
}

fn run_with_model<M: EvalModel>(
    features: M::Features,
    config: &Config,
    model_config: &M::Config,
    model_learn_config: &M::LearnConfig,
) -> Result<(), Box<dyn Error>> {
    let device = Device::cuda_if_available();
    log::info!("Learning using device: {device:?}");
    let mut vs = nn::VarStore::new(device);
    let mut model = M::new(features, vs.root(), model_config);
    if let Some(load_parameters) = &config.load_weights {
        vs.load(load_parameters)?;
    }
    let mut optimizer = nn::Adam::default().build(&vs, config.learning_rate)?;

    for epoch in 0..config.epochs {
        let mut num_samples = 0;
        let mut total_loss: f64 = 0.0;
        let start_time = Instant::now();
        let mut last_log_time = start_time;

        let mut dataset_iterator = DatasetIterator::new(&config.dataset)?;
        loop {
            let batch = dataset_iterator.next_batch()?;
            if batch.is_none() || last_log_time.elapsed().as_secs_f32() >= config.log_period_seconds
            {
                let elapsed_time = start_time.elapsed().as_secs_f64();
                log::info!(
                    "Epoch={epoch} / {num_epochs} samples={num_samples} time={elapsed_time:.2}s \
                    samples/s={samples_per_second:.0} loss={loss:.6}",
                    num_epochs = config.epochs,
                    samples_per_second = num_samples as f64 / elapsed_time,
                    loss = total_loss / num_samples as f64,
                );
                last_log_time = Instant::now();
            }
            let Some(batch) = batch else {
                break;
            };
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
            optimizer.backward_step(&loss);
            model.fixup(model_learn_config);
        }
    }
    vs.save(&config.save_weights)?;
    Ok(())
}
