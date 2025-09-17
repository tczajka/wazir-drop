use crate::{Color, Piece, Square};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OpeningMove {
    // From square 0 or square 63.
    pub pieces: [Piece; Self::SIZE],
}

impl OpeningMove {
    pub const SIZE: usize = 16;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColoredOpeningMove {
    pub color: Color,
    pub mov: OpeningMove,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RegularMove {
    pub piece: Piece,
    pub captured: Option<Piece>,
    pub from: Option<Square>,
    pub to: Square,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColoredRegularMove {
    pub color: Color,
    pub mov: RegularMove,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Move {
    Opening(OpeningMove),
    Regular(RegularMove),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColoredMove {
    pub color: Color,
    pub mov: Move,
}
