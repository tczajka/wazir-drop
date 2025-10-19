#![deny(unsafe_op_in_unsafe_fn)]
#![deny(unused_results)]
#![deny(clippy::default_trait_access)]

pub mod arrays;
pub mod enums;
pub mod parser;

mod bitboard;
mod color;
pub mod movegen;
mod moves;
mod piece;
mod position;
mod square;

pub use bitboard::{Bitboard, BitboardIterator};
pub use color::Color;
pub use moves::{InvalidMove, Move, OpeningMove, RegularMove, ShortMove, ShortMoveFrom};
pub use piece::{ColoredPiece, Piece, PieceNonWazir};
pub use position::{InvalidPosition, Position, Stage};
pub use square::{Coord, Direction, Square};
