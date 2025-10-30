use std::time::Duration;

pub const MAX_MOVES_IN_GAME: usize = 102;
pub const DEFAULT_TIME_LIMIT: Duration = Duration::from_secs(30);
pub const TIME_MARGIN: Duration = Duration::from_millis(200);

pub const MAX_SEARCH_DEPTH: u16 = 100;
pub const CHECK_TIMEOUT_NODES: u64 = 256;

#[derive(Debug, Clone)]
pub struct Hyperparameters {
    pub time_alloc_decay_moves: f64,
}

impl Default for Hyperparameters {
    fn default() -> Self {
        Self {
            time_alloc_decay_moves: 20.0,
        }
    }
}
