use std::{
    error::Error,
    fs::File,
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};

use rand::{SeedableRng, rngs::StdRng, seq::SliceRandom};
use serde::{Deserialize, Serialize};
use tch::{Device, Kind, Tensor};
use wazir_drop::constants::Eval;

use crate::config::FeaturesConfig;

#[derive(Debug, Serialize, Deserialize)]
pub struct Sample {
    /// [to move, other]
    pub features: [Vec<u16>; 2],
    /// Value from deeper search.
    // Eval::MAX is win, -Eval::MAX is loss
    pub deep_value: Eval,
    /// +1 = win, -1 = loss
    pub game_points: i32,
}

/// A batch of data.
pub struct Batch {
    pub size: usize,
    // Features: [num features in a batch]
    pub features: Tensor,
    // Offsets: [batch_size, 2] -> indices into features
    pub offsets: Tensor,
    // [batch_size] -> win probability
    pub outputs: Tensor,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetConfig {
    file: PathBuf,
    pub features: FeaturesConfig,
    value_scale: f32,
    chunk_size: usize,
    batch_size: usize,
    outcome_weight: f32,
}

impl Batch {
    pub fn to_device(&self, device: Device) -> Self {
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
                offsets.push(i32::try_from(features.len()).unwrap());
                features.extend(f.iter().map(|&f| i32::from(f)));
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

pub struct DatasetIterator {
    reader: BufReader<File>,
    buffer: Vec<u8>,
    input_value_scale: f32,
    outcome_weight: f32,
    chunk_size: usize,
    batch_size: usize,
    rng: StdRng,
    current_chunk: Vec<Sample>,
    current_chunk_index: usize,
}

impl DatasetIterator {
    pub fn new(config: &DatasetConfig) -> Result<Self, Box<dyn Error>> {
        let reader = BufReader::new(File::open(&config.file)?);
        Ok(Self {
            reader,
            buffer: vec![0; 1 << 10],
            input_value_scale: config.value_scale,
            outcome_weight: config.outcome_weight,
            chunk_size: config.chunk_size,
            batch_size: config.batch_size,
            rng: StdRng::from_os_rng(),
            current_chunk: Vec::with_capacity(config.chunk_size),
            current_chunk_index: 0,
        })
    }

    pub fn next_batch(&mut self) -> Result<Option<Batch>, Box<dyn Error>> {
        if self.current_chunk_index == self.current_chunk.len() {
            self.refill_chunk()?;
            if self.current_chunk_index == self.current_chunk.len() {
                return Ok(None);
            }
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

    fn refill_chunk(&mut self) -> Result<(), Box<dyn Error>> {
        self.current_chunk.clear();
        self.current_chunk_index = 0;
        while self.current_chunk.len() < self.chunk_size {
            match postcard::from_io((&mut self.reader, &mut self.buffer)) {
                Ok((sample, _)) => self.current_chunk.push(sample),
                Err(postcard::Error::DeserializeUnexpectedEnd) => break,
                Err(e) => return Err(e.into()),
            }
        }
        self.current_chunk.shuffle(&mut self.rng);
        Ok(())
    }
}

pub struct DatasetWriter {
    writer: BufWriter<File>,
}

impl DatasetWriter {
    pub fn new(filename: &Path) -> Result<Self, Box<dyn Error>> {
        let writer = BufWriter::new(File::create(filename)?);
        Ok(Self { writer })
    }

    pub fn write(&mut self, sample: &Sample) -> Result<(), Box<dyn Error>> {
        _ = postcard::to_io(sample, &mut self.writer)?;
        Ok(())
    }
}
