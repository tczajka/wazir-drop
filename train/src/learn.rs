use crate::{
    linear::LinearModel,
    model::EvalModel,
    self_play::{FeaturesConfig, Sample},
};
use rand::{SeedableRng, rngs::StdRng, seq::SliceRandom};
use serde::Deserialize;
use serde_cbor::Deserializer;
use std::{error::Error, fs::File, io::BufReader, path::PathBuf, time::Instant};
use tch::{
    Device, Reduction, Tensor, kind,
    nn::{self, OptimizerConfig},
};
use wazir_drop::{Features, PSFeatures};

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    self_play_data: PathBuf,
    input_value_scale: f32,
    features: FeaturesConfig,
    model: ModelConfig,
    learning_rate: f64,
    epochs: u32,
    batch_size: usize,
    outcome_weight: f32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelConfig {
    Linear,
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

fn run_with_model<M: EvalModel>(
    config: &Config,
    features: M::Features,
) -> Result<(), Box<dyn Error>> {
    let mut rng = StdRng::from_os_rng();
    let device = Device::cuda_if_available();
    log::info!("Using device: {device:?}");
    let mut dataset = Dataset::read_from_file(config, features)?;
    let vs = nn::VarStore::new(device);
    let model = M::new(features, vs.root());
    let mut optimizer = nn::AdamW::default().build(&vs, config.learning_rate)?;

    for epoch in 0..config.epochs {
        let mut num_examples = 0;
        let mut total_value_loss: f64 = 0.0;
        let mut total_outcome_loss: f64 = 0.0;
        let start_time = Instant::now();

        for batch in dataset.batches(config.batch_size, device, &mut rng) {
            let batch_size = batch.input.size()[0];
            let win_logits = model.forward(&batch.input);
            let win_prob = win_logits.sigmoid();
            let value_loss = win_prob.mse_loss(&batch.value_win_prob, Reduction::Sum);
            let outcome_loss = win_logits.binary_cross_entropy_with_logits::<Tensor>(
                &batch.actual_win,
                None,
                None,
                Reduction::Sum,
            );
            let loss =
                (1.0 - config.outcome_weight) * &value_loss + config.outcome_weight * &outcome_loss;

            num_examples += batch_size;
            total_value_loss += f64::try_from(&value_loss).unwrap();
            total_outcome_loss += f64::try_from(&outcome_loss).unwrap();

            optimizer.backward_step(&loss);
        }

        let elapsed_time = start_time.elapsed();
        log::info!(
            "Epoch {epoch} / {num_epochs} value loss {value_loss:.3} outcome loss {outcome_loss:.3} time {elapsed:.2}s examples/s {examples_per_second:.2}",
            num_epochs = config.epochs,
            value_loss = total_value_loss / num_examples as f64,
            outcome_loss = total_outcome_loss / num_examples as f64,
            elapsed = elapsed_time.as_secs_f64(),
            examples_per_second = num_examples as f64 / elapsed_time.as_secs_f64()
        );
    }
    Ok(())
}

/// One example or a batch.
struct Example {
    // Features: [2, N]
    input: Tensor,
    // []
    value_win_prob: Tensor,
    // []
    actual_win: Tensor,
}

impl Example {
    fn from_sample<F: Features>(sample: Sample, features: F, input_value_scale: f32) -> Self {
        let num_nonzero = sample.features[0].len() + sample.features[1].len();
        let mut inputs_player = Vec::with_capacity(num_nonzero);
        let mut inputs_features = Vec::with_capacity(num_nonzero);
        for player in 0..2 {
            for &feature in &sample.features[player] {
                inputs_player.push(player as i64);
                inputs_features.push(feature as i64);
            }
        }
        let input = Tensor::stack(
            &[
                Tensor::from_slice(&inputs_player),
                Tensor::from_slice(&inputs_features),
            ],
            0,
        );
        let input = Tensor::sparse_coo_tensor_indices_size(
            &input,
            &Tensor::ones(num_nonzero as i64, kind::FLOAT_CPU),
            [2, features.count() as i64],
            kind::FLOAT_CPU,
            false,
        );
        let value_win_prob = Tensor::from(sample.deep_value as f32 / input_value_scale).sigmoid();
        let actual_win = Tensor::from(sample.game_points as f32 * 0.5 + 0.5);
        Example {
            input,
            value_win_prob,
            actual_win,
        }
    }
}

struct Dataset<F: Features> {
    _features: F,
    examples: Vec<Example>,
}

impl<F: Features> Dataset<F> {
    fn read_from_file(config: &Config, features: F) -> Result<Self, Box<dyn Error>> {
        let input = BufReader::new(File::open(&config.self_play_data)?);
        let input = Deserializer::from_reader(input);
        let mut examples = Vec::new();
        for result_sample in input.into_iter() {
            let sample = result_sample?;
            let example = Example::from_sample(sample, features, config.input_value_scale);
            examples.push(example);
        }
        log::info!("Successfully read {len} samples", len = examples.len());
        Ok(Dataset {
            _features: features,
            examples,
        })
    }

    fn batches(
        &mut self,
        batch_size: usize,
        device: Device,
        rng: &mut StdRng,
    ) -> impl Iterator<Item = Example> {
        self.examples.shuffle(rng);
        self.examples.chunks(batch_size).map(move |chunk| {
            let input: Vec<Tensor> = chunk
                .iter()
                .map(|example| example.input.shallow_clone())
                .collect();
            let input = Tensor::stack(&input, 0);
            let value_win_prob: Vec<Tensor> = chunk
                .iter()
                .map(|example| example.value_win_prob.shallow_clone())
                .collect();
            let value_win_prob = Tensor::stack(&value_win_prob, 0);
            let actual_win: Vec<Tensor> = chunk
                .iter()
                .map(|example| example.actual_win.shallow_clone())
                .collect();
            let actual_win = Tensor::stack(&actual_win, 0);
            Example {
                input: input.to_device(device),
                value_win_prob: value_win_prob.to_device(device),
                actual_win: actual_win.to_device(device),
            }
        })
    }
}
