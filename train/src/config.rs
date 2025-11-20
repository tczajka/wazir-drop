use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
pub enum FeaturesConfig {
    PS,
    WPS,
}
