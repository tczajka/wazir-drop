use crate::{
    linear::LinearModel,
    model::EvalModel,
    self_play::{FeaturesConfig, Sample},
};
use rand::{SeedableRng, rngs::StdRng, seq::SliceRandom};
use serde::Deserialize;
use serde_cbor::{Deserializer, StreamDeserializer, de::IoRead};
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
    chunk_size: usize,
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
    let device = Device::cuda_if_available();
    log::info!("Using device: {device:?}");
    let vs = nn::VarStore::new(device);
    let model = M::new(features, vs.root());
    let mut optimizer = nn::AdamW::default().build(&vs, config.learning_rate)?;

    for epoch in 0..config.epochs {
        let mut num_examples = 0;
        let mut total_value_loss: f64 = 0.0;
        let mut total_outcome_loss: f64 = 0.0;
        let start_time = Instant::now();

        let mut dataset_iterator = DatasetIterator::new(config, features.count())?;
        while let Some(batch) = dataset_iterator.next_batch()? {
            let batch = batch.to_device(device);
            let values = model.forward(&batch.input);
            let win_prob = values.sigmoid();
            let value_loss = win_prob.mse_loss(&batch.values.sigmoid(), Reduction::Sum);
            let outcome_loss = values.binary_cross_entropy_with_logits::<Tensor>(
                &batch.outcomes,
                None,
                None,
                Reduction::Sum,
            );
            let loss =
                (1.0 - config.outcome_weight) * &value_loss + config.outcome_weight * &outcome_loss;

            num_examples += batch.size;
            total_value_loss += f64::try_from(&value_loss).unwrap();
            total_outcome_loss += f64::try_from(&outcome_loss).unwrap();

            optimizer.backward_step(&loss);
        }

        let elapsed_time = start_time.elapsed();
        log::info!(
            "Epoch {epoch} / {num_epochs} examples {num_examples} time {elapsed:.2}s examples/s {examples_per_second:.0}\n  \
            value loss {value_loss:.3} outcome loss {outcome_loss:.3}",
            num_epochs = config.epochs,
            value_loss = total_value_loss / num_examples as f64,
            outcome_loss = total_outcome_loss / num_examples as f64,
            elapsed = elapsed_time.as_secs_f64(),
            examples_per_second = num_examples as f64 / elapsed_time.as_secs_f64()
        );
    }
    Ok(())
}

/// A batch of data.
struct Batch {
    size: i64,
    // Features: [batch_size,2, N]
    input: Tensor,
    // [batch_size]
    values: Tensor,
    // [batch_size]
    outcomes: Tensor,
}

impl Batch {
    fn to_device(&self, device: Device) -> Self {
        Self {
            size: self.size,
            input: self.input.to_device(device),
            values: self.values.to_device(device),
            outcomes: self.outcomes.to_device(device),
        }
    }

    fn from_samples(samples: &[Sample], num_features: usize, input_value_scale: f32) -> Self {
        let mut inputs = Vec::with_capacity(samples.len());
        let mut values = Vec::with_capacity(samples.len());
        let mut outcomes = Vec::with_capacity(samples.len());
        for sample in samples {
            let feature_tensors: [Tensor; 2] = sample.features.each_ref().map(|features| {
                let features: Vec<i64> = features.iter().map(|&feature| feature as i64).collect();
                Tensor::sparse_coo_tensor_indices_size(
                    &Tensor::from_slice(&features).unsqueeze(0),
                    &Tensor::ones(features.len() as i64, kind::FLOAT_CPU),
                    [num_features as i64],
                    kind::FLOAT_CPU,
                    false,
                )
            });
            inputs.push(Tensor::stack(&feature_tensors, 0));
            let value = sample.deep_value as f32 / input_value_scale;
            values.push(value);
            let outcome = sample.game_points as f32 * 0.5 + 0.5;
            outcomes.push(outcome);
        }
        Self {
            size: samples.len() as i64,
            input: Tensor::stack(&inputs, 0),
            values: Tensor::from_slice(&values),
            outcomes: Tensor::from_slice(&outcomes),
        }
    }
}

struct DatasetIterator<'de> {
    /// None if the whole dataset is already loaded in memory.
    deserializer: Option<StreamDeserializer<'de, IoRead<BufReader<File>>, Sample>>,
    input_value_scale: f32,
    num_features: usize,
    chunk_size: usize,
    batch_size: usize,
    rng: StdRng,
    current_chunk: Vec<Sample>,
    current_chunk_index: usize,
}

impl<'de> DatasetIterator<'de> {
    fn new(config: &Config, num_features: usize) -> Result<Self, Box<dyn Error>> {
        let input = BufReader::new(File::open(&config.self_play_data)?);
        let deserializer = Deserializer::from_reader(input);
        Ok(Self {
            deserializer: Some(deserializer.into_iter()),
            input_value_scale: config.input_value_scale,
            num_features,
            chunk_size: config.chunk_size,
            batch_size: config.batch_size,
            rng: StdRng::from_os_rng(),
            current_chunk: Vec::with_capacity(config.chunk_size),
            current_chunk_index: 0,
        })
    }

    fn next_batch(&mut self) -> Result<Option<Batch>, Box<dyn Error>> {
        if self.current_chunk_index == self.current_chunk.len() && !self.refill_chunk()? {
            return Ok(None);
        }
        let next_chunk_index =
            (self.current_chunk_index + self.batch_size).min(self.current_chunk.len());
        let samples = &self.current_chunk[self.current_chunk_index..next_chunk_index];
        self.current_chunk_index = next_chunk_index;
        Ok(Some(Batch::from_samples(
            samples,
            self.num_features,
            self.input_value_scale,
        )))
    }

    fn refill_chunk(&mut self) -> Result<bool, Box<dyn Error>> {
        let Some(deserializer) = &mut self.deserializer else {
            return Ok(false);
        };
        self.current_chunk.clear();
        self.current_chunk_index = 0;
        while self.current_chunk.len() < self.chunk_size {
            let Some(result) = deserializer.next() else {
                self.deserializer = None;
                break;
            };
            let sample = result?;
            self.current_chunk.push(sample);
        }
        if self.current_chunk.is_empty() {
            return Ok(false);
        }
        self.current_chunk.shuffle(&mut self.rng);
        log::info!("next chunk: {}", self.current_chunk.len());
        Ok(true)
    }
}
