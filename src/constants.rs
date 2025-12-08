use std::time::Duration;

pub const DEFAULT_TIME_LIMIT: Duration = Duration::from_secs(30);
pub const TIME_MARGIN: Duration = Duration::from_millis(200);
pub const MAX_VARIATION_LENGTH: usize = 100;
pub const CHECK_TIMEOUT_NODES: u64 = 256;

pub type Ply = u8;
pub const PLY_AFTER_SETUP: Ply = 2;
pub const PLY_DRAW: Ply = 102;

pub type Depth = u16;
pub const ONE_PLY: Depth = 100;
pub const MAX_SEARCH_DEPTH: Depth = 100 * ONE_PLY;
pub const DEPTH_INCREMENT: Depth = ONE_PLY;

pub type Eval = i32;

pub const NUM_KILLER_MOVES: usize = 2;

// 16 KB, 2 hashes: collision probability per hash is 1 / 80, total 1 / 6400
pub const HISTORY_BLOOM_FILTER_LOG_SIZE: u32 = 14;
pub const HISTORY_BLOOM_FILTER_NUM_HASHES: usize = 2;

#[derive(Debug, Clone)]
pub struct Hyperparameters {
    pub ttable_size: usize,
    pub pvtable_size: usize,
    pub time_alloc_decay_moves: f64,
    pub min_depth_ttable: Depth,
    pub reduction_null_move: Depth,
    pub futility_margin: f32,
}

impl Default for Hyperparameters {
    fn default() -> Self {
        Self {
            ttable_size: 256 << 20,
            pvtable_size: 16 << 20,
            time_alloc_decay_moves: 20.0,
            min_depth_ttable: 200,
            reduction_null_move: 100,
            futility_margin: 0.8,
        }
    }
}
