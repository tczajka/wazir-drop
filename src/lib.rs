#![deny(unsafe_op_in_unsafe_fn)]
#![deny(unused_results)]
#![deny(clippy::default_trait_access)]

pub mod array;
pub mod either;
pub mod enum_map;
pub mod parser;

mod bitboard;
mod color;
mod mov;
mod piece;
mod position;
mod square;

pub use bitboard::Bitboard;
pub use color::Color;
pub use mov::{ColoredMove, ColoredOpeningMove, ColoredRegularMove, OpeningMove, RegularMove};
pub use piece::{ColoredPiece, Piece};
pub use position::Position;
pub use square::{Coord, Square, Vector};
