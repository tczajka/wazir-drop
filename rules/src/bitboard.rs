use crate::{Square, BOARD_HEIGHT, BOARD_WIDTH};
use std::{
    fmt::{self, Display, Formatter},
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Bitboard(u64);

impl Bitboard {
    pub const EMPTY: Self = Self(0);

    pub const fn single(square: Square) -> Self {
        Self(1 << square.index())
    }

    pub fn contains(&self, square: Square) -> bool {
        *self & Self::single(square) != Self::EMPTY
    }

    pub fn add(&mut self, square: Square) {
        *self = *self | Self::single(square);
    }

    pub fn remove(&mut self, square: Square) {
        *self = *self & !Self::single(square);
    }
}

impl BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, other: Self) -> Self::Output {
        Self(self.0 | other.0)
    }
}

impl BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, other: Self) -> Self::Output {
        Self(self.0 & other.0)
    }
}

impl BitXor for Bitboard {
    type Output = Self;

    fn bitxor(self, other: Self) -> Self::Output {
        Self(self.0 ^ other.0)
    }
}

impl Not for Bitboard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, other: Self) {
        *self = *self & other;
    }
}

impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, other: Self) {
        *self = *self | other;
    }
}

impl BitXorAssign for Bitboard {
    fn bitxor_assign(&mut self, other: Self) {
        *self = *self ^ other;
    }
}

impl Display for Bitboard {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for row in 0..BOARD_HEIGHT {
            for column in 0..BOARD_WIDTH {
                let square = Square::from_row_column(row, column);
                if self.contains(square) {
                    write!(f, "x")?;
                } else {
                    write!(f, ".")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
