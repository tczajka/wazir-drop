#![deny(unsafe_op_in_unsafe_fn)]
#![deny(unused_results)]
#![deny(clippy::default_trait_access)]

pub mod arrays;
pub mod clock;
pub mod either;
pub mod enums;
pub mod error;
pub mod parser;
pub mod smallvec;

mod bitboard;
mod board;
mod captured;
mod color;
pub mod movegen;
mod moves;
mod piece;
mod player;
mod position;
mod square;

pub use bitboard::{Bitboard, BitboardIterator};
pub use board::Board;
pub use captured::Captured;
pub use color::Color;
pub use moves::{
    InvalidMove, Move, RegularMove, SetupMove, ShortMove, ShortMoveFrom, MAX_MOVES_IN_GAME,
};
pub use piece::{ColoredPiece, Piece};
pub use player::Player;
pub use position::{Position, Stage};
pub use square::{Coord, Direction, Square};
