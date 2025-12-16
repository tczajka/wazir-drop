pub mod arrays;
pub mod base128;
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
pub mod book;
mod book_data;
mod captured;
mod cli;
mod color;
pub mod constants;
mod eval;
mod features;
mod history;
mod main_player;
pub mod movegen;
mod moves;
mod nnue;
mod nnue_weights;
mod piece;
mod player;
mod position;
mod pvtable;
mod score;
mod search;
mod square;
mod symmetry;
mod ttable;
mod variation;
pub mod vector;
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
pub use history::History;
pub use main_player::MainPlayerFactory;
pub use moves::{AnyMove, InvalidMove, Move, SetupMove, ShortMove, ShortMoveFrom};
pub use nnue::Nnue;
pub use piece::{ColoredPiece, Piece};
pub use player::{Player, PlayerFactory};
pub use position::{Outcome, Position, Stage};
pub use pvtable::PVTable;
pub use score::{Score, ScoreExpanded};
pub use search::{Deadlines, ScoredMove, Search};
pub use square::{Coord, Direction, Square};
pub use symmetry::{NormalizedSquare, Symmetry};
pub use variation::{
    EmptyVariation, ExtendableVariation, LongVariation, NonEmptyVariation, OneMoveVariation,
    Variation,
};
pub use wps_features::WPSFeatures;

pub type DefaultEvaluator = Nnue;
