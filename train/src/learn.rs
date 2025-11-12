use crate::{
    linear::{self, LinearModel},
    model::EvalModel,
    nnue::{self, NnueModel},
    self_play::{FeaturesConfig, Sample},
};
use extra::PSFeatures;
use rand::{SeedableRng, rngs::StdRng, seq::SliceRandom};
use serde::Deserialize;
use serde_cbor::{Deserializer, StreamDeserializer, de::IoRead};
use std::{error::Error, fs::File, io::BufReader, path::PathBuf, time::Instant};
use tch::{Device, Kind, Reduction, Tensor, nn};
use wazir_drop::{Features, WPSFeatures};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    self_play_data: PathBuf,
    load_weights: Option<PathBuf>,
    save_weights: PathBuf,
    input_value_scale: f32,
    features: FeaturesConfig,
    model: ModelConfig,
    epochs: u32,
    chunk_size: usize,
    batch_size: usize,
    outcome_weight: f32,
    log_period_seconds: f32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelConfig {
    Linear(linear::Config),
    Nnue(nnue::Config),
}

pub fn run(config: &Config) -> Result<(), Box<dyn Error>> {
    match config.features {
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
    log::info!("Learning using device: {device:?}");
    let mut vs = nn::VarStore::new(device);
    let mut model = M::new(features, vs.root(), model_config);
    if let Some(load_parameters) = &config.load_weights {
        vs.load(load_parameters)?;
    }
    let mut optimizer = model.optimizer(&vs)?;

    for epoch in 0..config.epochs {
        let mut num_samples = 0;
        let mut total_loss: f64 = 0.0;
        let start_time = Instant::now();
        let mut last_log_time = start_time;

        let mut dataset_iterator = DatasetIterator::new(config)?;
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
            model.fixup();
        }
    }
    vs.save(&config.save_weights)?;
    Ok(())
}

/// A batch of data.
struct Batch {
    size: usize,
    // Features: [num features in a batch]
    features: Tensor,
    // Offsets: [batch_size, 2] -> indices into features
    offsets: Tensor,
    // [batch_size] -> win probability
    outputs: Tensor,
}

impl Batch {
    fn to_device(&self, device: Device) -> Self {
        Self {
            size: self.size,
            features: self.features.to_device(device),
            offsets: self.offsets.to_device(device),
            outputs: self.outputs.to_device(device),
        }
    }

    fn from_samples(samples: &[Sample], input_value_scale: f32, outcome_weight: f32) -> Self {
        let mut features = Vec::new();
        let mut offsets = Vec::with_capacity(samples.len() * 2);
        let mut values = Vec::with_capacity(samples.len());
        let mut outcomes = Vec::with_capacity(samples.len());
        for sample in samples {
            for f in &sample.features {
                offsets.push(features.len() as i32);
                features.extend(f.iter().map(|&f| f as i32));
            }
            values.push(sample.deep_value);
            outcomes.push(sample.game_points);
        }
        let features = Tensor::from_slice(&features).to_kind(Kind::Int64);
        let offsets = Tensor::from_slice(&offsets)
            .reshape([-1, 2])
            .to_kind(Kind::Int64);
        let values = (1.0 / input_value_scale * Tensor::from_slice(&values).to_kind(Kind::Float))
            .sigmoid()
            .to_kind(Kind::Float);
        let outcomes = 0.5
            + 0.5
                * Tensor::from_slice(&outcomes)
                    .to_kind(Kind::Float)
                    .to_kind(Kind::Float);
        let outputs = (1.0 - outcome_weight) * values + outcome_weight * outcomes;
        Self {
            size: samples.len(),
            features,
            offsets,
            outputs,
        }
    }
}

struct DatasetIterator<'de> {
    /// None if the whole dataset is already loaded in memory.
    deserializer: Option<StreamDeserializer<'de, IoRead<BufReader<File>>, Sample>>,
    input_value_scale: f32,
    outcome_weight: f32,
    chunk_size: usize,
    batch_size: usize,
    rng: StdRng,
    current_chunk: Vec<Sample>,
    current_chunk_index: usize,
}

impl<'de> DatasetIterator<'de> {
    fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let input = BufReader::new(File::open(&config.self_play_data)?);
        let deserializer = Deserializer::from_reader(input);
        Ok(Self {
            deserializer: Some(deserializer.into_iter()),
            input_value_scale: config.input_value_scale,
            outcome_weight: config.outcome_weight,
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
            self.input_value_scale,
            self.outcome_weight,
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
        Ok(true)
    }
}
