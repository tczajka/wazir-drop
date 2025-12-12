use std::time::Duration;

pub const DEFAULT_TIME_LIMIT: Duration = Duration::from_secs(30);
pub const TIME_MARGIN: Duration = Duration::from_millis(300);
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

pub const RED_SETUP_INDEX: usize = 10;

#[derive(Debug, Clone)]
pub struct Hyperparameters {
    pub ttable_size: usize,
    pub pvtable_size: usize,
    pub min_depth_ttable: Depth,
    pub reduction_null_move: Depth,
    pub futility_margin: f32,
    pub late_move_reduction_start: usize,
    pub time_reduction_per_move: f64,
    pub time_reduction_per_late_move: f64,
    pub late_ply: Ply,
    pub soft_time_fraction: f64,
    pub start_next_depth_fraction: f64,
    pub panic_eval_threshold: f64,
    pub panic_multiplier: f64,
    pub panic_max_remaining: f64,
}

impl Default for Hyperparameters {
    fn default() -> Self {
        Self {
            ttable_size: 256 << 20,
            pvtable_size: 16 << 20,
            min_depth_ttable: 2 * ONE_PLY,
            reduction_null_move: ONE_PLY,
            futility_margin: 0.8,
            late_move_reduction_start: 5,
            time_reduction_per_move: 0.05,
            time_reduction_per_late_move: 0.5,
            late_ply: 96,
            soft_time_fraction: 0.8,
            start_next_depth_fraction: 0.4,
            panic_eval_threshold: 0.1,
            panic_multiplier: 2.0,
            panic_max_remaining: 0.3,
        }
    }
}
