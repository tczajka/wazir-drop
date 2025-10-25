use crate::{enums::EnumMap, unsafe_simple_enum, Coord, Square};

/// Apply FlipX, FlipY and SwapXY in that order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Symmetry {
    Identity,
    FlipX,
    FlipY,
    Rotate180,
    SwapXY,
    RotateLeft,
    RotateRight,
    OtherDiagonal,
}

unsafe_simple_enum!(Symmetry, 8);

impl Symmetry {
    pub const fn from_bits(flip_x: bool, flip_y: bool, swap_xy: bool) -> Self {
        Self::from_index(flip_x as usize | (flip_y as usize) << 1 | (swap_xy as usize) << 2)
    }

    // Returns (flip_x, flip_y, swap_xy)
    pub const fn to_bits(self) -> (bool, bool, bool) {
        let bits = self.index();
        (bits & 1 != 0, bits & 2 != 0, bits & 4 != 0)
    }

    pub fn inverse(self) -> Self {
        INVERSE_TABLE[self]
    }

    pub fn apply(self, square: Square) -> Square {
        APPLY_TABLE[self][square]
    }

    pub fn normalize(square: Square) -> (Self, NormalizedSquare) {
        NORMALIZE_TABLE[square]
    }
}

#[rustfmt::skip]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum NormalizedSquare {
    A1, A2, A3, A4,
        B2, B3, B4,
            C3, C4,
                D4,
}

unsafe_simple_enum!(NormalizedSquare, 10);

impl NormalizedSquare {
    pub const fn try_from_square(square: Square) -> Option<Self> {
        match square {
            Square::A1 => Some(Self::A1),
            Square::A2 => Some(Self::A2),
            Square::A3 => Some(Self::A3),
            Square::A4 => Some(Self::A4),
            Square::B2 => Some(Self::B2),
            Square::B3 => Some(Self::B3),
            Square::B4 => Some(Self::B4),
            Square::C3 => Some(Self::C3),
            Square::C4 => Some(Self::C4),
            Square::D4 => Some(Self::D4),
            _ => None,
        }
    }
}

impl From<NormalizedSquare> for Square {
    fn from(normalized_square: NormalizedSquare) -> Square {
        match normalized_square {
            NormalizedSquare::A1 => Square::A1,
            NormalizedSquare::A2 => Square::A2,
            NormalizedSquare::A3 => Square::A3,
            NormalizedSquare::A4 => Square::A4,
            NormalizedSquare::B2 => Square::B2,
            NormalizedSquare::B3 => Square::B3,
            NormalizedSquare::B4 => Square::B4,
            NormalizedSquare::C3 => Square::C3,
            NormalizedSquare::C4 => Square::C4,
            NormalizedSquare::D4 => Square::D4,
        }
    }
}

static INVERSE_TABLE: EnumMap<Symmetry, Symmetry> = compute_inverse_table();

const fn compute_inverse_table() -> EnumMap<Symmetry, Symmetry> {
    let mut table = [Symmetry::Identity; Symmetry::COUNT];
    let mut symmetry_idx = 0;
    while symmetry_idx != Symmetry::COUNT {
        table[symmetry_idx] = compute_inverse(Symmetry::from_index(symmetry_idx));
        symmetry_idx += 1;
    }
    EnumMap::from_array(table)
}

const fn compute_inverse(symmetry: Symmetry) -> Symmetry {
    let (mut flip_x, mut flip_y, swap_xy) = symmetry.to_bits();
    if swap_xy {
        (flip_x, flip_y) = (flip_y, flip_x);
    }
    Symmetry::from_bits(flip_x, flip_y, swap_xy)
}

static APPLY_TABLE: EnumMap<Symmetry, EnumMap<Square, Square>> = compute_apply_table();

const fn compute_apply_table() -> EnumMap<Symmetry, EnumMap<Square, Square>> {
    let mut table = [EnumMap::from_array([Square::A1; Square::COUNT]); Symmetry::COUNT];
    let mut symmetry_idx = 0;
    while symmetry_idx != Symmetry::COUNT {
        table[symmetry_idx] = compute_apply_table_for_symmetry(Symmetry::from_index(symmetry_idx));
        symmetry_idx += 1;
    }
    EnumMap::from_array(table)
}

const fn compute_apply_table_for_symmetry(symmetry: Symmetry) -> EnumMap<Square, Square> {
    let mut table = [Square::A1; Square::COUNT];
    let mut square_idx = 0;
    while square_idx != Square::COUNT {
        table[square_idx] = compute_apply(symmetry, Square::from_index(square_idx));
        square_idx += 1;
    }
    EnumMap::from_array(table)
}

const fn compute_apply(symmetry: Symmetry, square: Square) -> Square {
    let (flip_x, flip_y, swap_xy) = symmetry.to_bits();
    let coord = Coord::from_square(square);
    let mut x = coord.x();
    let mut y = coord.y();
    if flip_x {
        x = Coord::WIDTH - 1 - x;
    }
    if flip_y {
        y = Coord::HEIGHT - 1 - y;
    }
    if swap_xy {
        (x, y) = (y, x);
    }
    Square::from_coord(Coord::new(x, y))
}

static NORMALIZE_TABLE: EnumMap<Square, (Symmetry, NormalizedSquare)> = compute_normalize_table();

const fn compute_normalize_table() -> EnumMap<Square, (Symmetry, NormalizedSquare)> {
    let mut table = [(Symmetry::Identity, NormalizedSquare::A1); Square::COUNT];
    let mut square_idx = 0;
    while square_idx != Square::COUNT {
        table[square_idx] = compute_normalize(Square::from_index(square_idx));
        square_idx += 1;
    }
    EnumMap::from_array(table)
}

const fn compute_normalize(square: Square) -> (Symmetry, NormalizedSquare) {
    let mut symmetry_idx = 0;
    while symmetry_idx != Symmetry::COUNT {
        let symmetry = Symmetry::from_index(symmetry_idx);
        if let Some(normalized_square) =
            NormalizedSquare::try_from_square(compute_apply(symmetry, square))
        {
            return (symmetry, normalized_square);
        }
        symmetry_idx += 1;
    }
    unreachable!()
}
