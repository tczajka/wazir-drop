use serde::Deserialize;
use serde_cbor::Deserializer;
use std::{
    error::Error,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use crate::self_play::Sample;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    self_play_data: PathBuf,
}

pub fn run(config: &Config) -> Result<(), Box<dyn Error>> {
    let dataset = Dataset::from_file(&config.self_play_data)?;
    Ok(())
}

struct Dataset {
    samples: Vec<Sample>,
}

impl Dataset {
    fn from_file(file_name: &Path) -> Result<Self, Box<dyn Error>> {
        let input = BufReader::new(File::open(file_name)?);
        let input = Deserializer::from_reader(input);
        let samples: Result<Vec<Sample>, _> = input.into_iter().collect();
        let samples = samples?;
        log::info!("Successfully read {len} samples", len = samples.len());
        Ok(Dataset { samples })
    }
}
