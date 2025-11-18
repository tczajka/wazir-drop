use serde::Deserialize;
use crate::{linear, nnue};

#[derive(Clone, Debug, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
pub enum FeaturesConfig {
    PS,
    WPS,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelConfig {
    Linear(linear::Config),
    Nnue(nnue::Config),
}