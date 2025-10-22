#![deny(unsafe_op_in_unsafe_fn)]
#![deny(unused_results)]
#![deny(clippy::default_trait_access)]

pub mod arrays;
pub mod enums;
pub mod parser;
pub mod smallvec;

mod bitboard;
mod board;
mod color;
pub mod movegen;
mod moves;
mod piece;
mod position;
mod square;

pub use bitboard::{Bitboard, BitboardIterator};
pub use color::Color;
pub use moves::{InvalidMove, Move, RegularMove, SetupMove, ShortMove, ShortMoveFrom};
pub use piece::{ColoredPiece, Piece};
pub use position::{InvalidPosition, Position, Stage};
pub use square::{Coord, Direction, Square};
