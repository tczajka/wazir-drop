pub mod arrays;
pub mod clock;
pub mod either;
pub mod enums;
pub mod error;
pub mod log;
pub mod parser;
pub mod platform;
pub mod smallvec;

mod bitboard;
mod board;
mod captured;
mod cli;
mod color;
pub mod constants;
mod eval;
mod features;
mod linear_eval;
pub mod linear_ps_weights;
mod main_player;
pub mod movegen;
mod moves;
mod piece;
mod player;
mod position;
mod ps_features;
mod score;
mod search;
mod square;
mod symmetry;
mod ttable;
mod wps_features;
mod zobrist;

#[cfg(test)]
mod tests;

pub use bitboard::{Bitboard, BitboardIterator};
pub use board::Board;
pub use captured::{captured_index, Captured, CapturedOneSide, NUM_CAPTURED_INDEXES};
pub use cli::{run_cli, CliCommand};
pub use color::Color;
pub use eval::{EvaluatedPosition, Evaluator};
pub use features::Features;
pub use linear_eval::LinearEvaluator;
pub use main_player::MainPlayerFactory;
pub use moves::{InvalidMove, Move, RegularMove, SetupMove, ShortMove, ShortMoveFrom};
pub use piece::{ColoredPiece, Piece};
pub use player::{Player, PlayerFactory};
pub use position::{Outcome, Position, Stage};
pub use ps_features::PSFeatures;
pub use score::{Score, ScoreExpanded};
pub use search::{Search, TopVariation, Variation};
pub use square::{Coord, Direction, Square};
pub use symmetry::{NormalizedSquare, Symmetry};
pub use wps_features::WPSFeatures;

pub type DefaultEvaluator = LinearEvaluator<PSFeatures>;
