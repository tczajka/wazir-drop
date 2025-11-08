use std::time::Duration;

pub const DEFAULT_TIME_LIMIT: Duration = Duration::from_secs(30);
pub const TIME_MARGIN: Duration = Duration::from_millis(200);
pub const MAX_VARIATION_LENGTH: usize = 100;
pub const CHECK_TIMEOUT_NODES: u64 = 256;

pub type MoveNumber = u8;
pub const MOVE_NUMBER_DRAW: MoveNumber = 102;

pub type Depth = i16;
pub const MAX_SEARCH_DEPTH: Depth = 100;
pub const INFINITE_DEPTH: Depth = i16::MAX;

pub type Eval = i32;

#[derive(Debug, Clone)]
pub struct Hyperparameters {
    pub ttable_size: usize,
    pub pvtable_size: usize,
    pub time_alloc_decay_moves: f64,
    pub min_depth_ttable: Depth,
    // In addition to 1 ply.
    pub reduction_null_move: Depth,
}

impl Default for Hyperparameters {
    fn default() -> Self {
        Self {
            ttable_size: 1024 << 20,
            pvtable_size: 16 << 20,
            time_alloc_decay_moves: 20.0,
            min_depth_ttable: 2,
            reduction_null_move: 1,
        }
    }
}
