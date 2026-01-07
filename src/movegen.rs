use crate::{
    either::Either, enums::SimpleEnumExt, smallvec::SmallVec, AnyMove, Bitboard, Color,
    InvalidMove, Move, Piece, Position, SetupMove, ShortMove, ShortMoveFrom, Square, Stage,
};
use std::iter;

pub fn move_bitboard(piece: Piece, square: Square) -> Bitboard {
    MOVE_TABLE[piece.index()][square.index()]
}

/// Includes going back to the same square.
pub fn double_move_bitboard(piece: Piece, square: Square) -> Bitboard {
    DOUBLE_MOVE_TABLE[piece.index()][square.index()]
}

/// Includes single moves.
pub fn triple_move_bitboard(piece: Piece, square: Square) -> Bitboard {
    TRIPLE_MOVE_TABLE[piece.index()][square.index()]
}

pub fn wazir_plus_move_bitboard(piece: Piece, square: Square) -> Bitboard {
    WAZIR_PLUS_MOVE_TABLE[piece.index()][square.index()]
}

pub fn wazir_plus_double_move_bitboard(piece: Piece, square: Square) -> Bitboard {
    WAZIR_PLUS_DOUBLE_MOVE_TABLE[piece.index()][square.index()]
}

pub fn validate_from_to(piece: Piece, from: Square, to: Square) -> Result<(), InvalidMove> {
    if !move_bitboard(piece, from).contains(to) {
        return Err(InvalidMove);
    }
    Ok(())
}

const CONST_NO_MOVE_TABLE: [[Bitboard; Square::COUNT]; Piece::COUNT] = {
    let mut table = [Bitboard::EMPTY; Square::COUNT];
    let mut square_idx = 0;
    while square_idx != Square::COUNT {
        table[square_idx] = Bitboard::single(Square::from_index(square_idx));
        square_idx += 1;
    }
    [table; Piece::COUNT]
};

const fn apply_move_by_piece(
    table: [Bitboard; Square::COUNT],
    piece: Piece,
) -> [Bitboard; Square::COUNT] {
    let mut new_table = [Bitboard::EMPTY; Square::COUNT];
    let directions = piece.directions();
    let mut square_idx = 0;
    while square_idx != Square::COUNT {
        let square = Square::from_index(square_idx);
        let mut bitboard = Bitboard::EMPTY;
        let mut i = 0;
        while i != directions.len() {
            if let Some(square2) = square.add(directions[i]) {
                bitboard = bitboard.union(table[square2.index()]);
            }
            i += 1;
        }
        new_table[square_idx] = bitboard;
        square_idx += 1;
    }
    new_table
}

const fn apply_move(
    table: [[Bitboard; Square::COUNT]; Piece::COUNT],
) -> [[Bitboard; Square::COUNT]; Piece::COUNT] {
    let mut new_table = [[Bitboard::EMPTY; Square::COUNT]; Piece::COUNT];
    let mut piece_idx = 0;
    while piece_idx != Piece::COUNT {
        new_table[piece_idx] = apply_move_by_piece(table[piece_idx], Piece::from_index(piece_idx));
        piece_idx += 1;
    }
    new_table
}

const fn apply_wazir_move(
    table: [[Bitboard; Square::COUNT]; Piece::COUNT],
) -> [[Bitboard; Square::COUNT]; Piece::COUNT] {
    let mut new_table = [[Bitboard::EMPTY; Square::COUNT]; Piece::COUNT];
    let mut piece_idx = 0;
    while piece_idx != Piece::COUNT {
        new_table[piece_idx] = apply_move_by_piece(table[piece_idx], Piece::Wazir);
        piece_idx += 1;
    }
    new_table
}

const CONST_MOVE_TABLE: [[Bitboard; Square::COUNT]; Piece::COUNT] = apply_move(CONST_NO_MOVE_TABLE);

const CONST_DOUBLE_MOVE_TABLE: [[Bitboard; Square::COUNT]; Piece::COUNT] =
    apply_move(CONST_MOVE_TABLE);

const CONST_TRIPLE_MOVE_TABLE: [[Bitboard; Square::COUNT]; Piece::COUNT] =
    apply_move(CONST_DOUBLE_MOVE_TABLE);

const CONST_WAZIR_PLUS_MOVE_TABLE: [[Bitboard; Square::COUNT]; Piece::COUNT] =
    apply_wazir_move(CONST_MOVE_TABLE);

const CONST_WAZIR_PLUS_DOUBLE_MOVE_TABLE: [[Bitboard; Square::COUNT]; Piece::COUNT] =
    apply_wazir_move(CONST_DOUBLE_MOVE_TABLE);

static MOVE_TABLE: [[Bitboard; Square::COUNT]; Piece::COUNT] = CONST_MOVE_TABLE;

static DOUBLE_MOVE_TABLE: [[Bitboard; Square::COUNT]; Piece::COUNT] = CONST_DOUBLE_MOVE_TABLE;

static TRIPLE_MOVE_TABLE: [[Bitboard; Square::COUNT]; Piece::COUNT] = CONST_TRIPLE_MOVE_TABLE;

static WAZIR_PLUS_MOVE_TABLE: [[Bitboard; Square::COUNT]; Piece::COUNT] =
    CONST_WAZIR_PLUS_MOVE_TABLE;

static WAZIR_PLUS_DOUBLE_MOVE_TABLE: [[Bitboard; Square::COUNT]; Piece::COUNT] =
    CONST_WAZIR_PLUS_DOUBLE_MOVE_TABLE;

pub fn any_move_from_short_move(
    position: &Position,
    short_move: ShortMove,
) -> Result<AnyMove, InvalidMove> {
    match short_move {
        ShortMove::Setup(mov) => {
            if position.stage() != Stage::Setup || mov.color != position.to_move() {
                return Err(InvalidMove);
            }
            mov.validate_pieces()?;
            Ok(AnyMove::Setup(mov))
        }
        ShortMove::Regular { from, to } => {
            if position.stage() != Stage::Regular {
                return Err(InvalidMove);
            }

            let captured = match position.square(to) {
                None => None,
                Some(captured) => {
                    if captured.color() != position.to_move().opposite() {
                        return Err(InvalidMove);
                    }
                    Some(captured.piece())
                }
            };

            let (colored_piece, from) = match from {
                ShortMoveFrom::Piece(cpiece) => {
                    if captured.is_some() || position.num_captured(cpiece) == 0 {
                        return Err(InvalidMove);
                    }
                    (cpiece, None)
                }
                ShortMoveFrom::Square(square) => {
                    let piece = position.square(square).ok_or(InvalidMove)?;
                    validate_from_to(piece.piece(), square, to)?;
                    (piece, Some(square))
                }
            };

            if colored_piece.color() != position.to_move() {
                return Err(InvalidMove);
            }
            Ok(AnyMove::Regular(Move {
                colored_piece,
                from,
                captured,
                to,
            }))
        }
    }
}

pub fn setup_moves(color: Color) -> impl Iterator<Item = SetupMove> {
    SetupMoveIterator { color, mov: None }
}

#[derive(Debug)]
struct SetupMoveIterator {
    color: Color,
    mov: Option<SetupMove>,
}

impl Iterator for SetupMoveIterator {
    type Item = SetupMove;

    fn next(&mut self) -> Option<Self::Item> {
        match self.mov {
            None => {
                #[allow(clippy::manual_repeat_n)]
                let pieces: SmallVec<Piece, { SetupMove::SIZE }> = Piece::all()
                    .flat_map(|piece| iter::repeat(piece).take(piece.initial_count()))
                    .collect();
                let pieces = (&pieces[..]).try_into().unwrap();
                self.mov = Some(SetupMove {
                    color: self.color,
                    pieces,
                });
            }
            Some(ref mut mov) => {
                let mut i = SetupMove::SIZE - 1;
                loop {
                    // mov.pieces[i..] is in non-ascending order
                    if i == 0 {
                        return None;
                    }
                    i -= 1;
                    if mov.pieces[i] < mov.pieces[i + 1] {
                        break;
                    }
                }
                // mov.pieces[i] < mov.pieces[i+1] >= ...
                let mut j = i + 1;
                while j != SetupMove::SIZE - 1 && mov.pieces[i] < mov.pieces[j + 1] {
                    j += 1;
                }
                // mov.pieces[i] < mov.pieces[j]
                // mov.pieces[i] >= mov.pieces[j+1]
                mov.pieces.swap(i, j);
                mov.pieces[i + 1..].reverse();
                self.mov = Some(*mov);
            }
        }
        self.mov
    }
}

pub fn attacked_by(position: &Position, square: Square, color: Color) -> Bitboard {
    let mut res = Bitboard::EMPTY;
    for piece in Piece::all() {
        res |= move_bitboard(piece, square) & position.occupied_by_piece(piece.with_color(color));
    }
    res
}

pub fn is_attacked_by(position: &Position, square: Square, color: Color) -> bool {
    !attacked_by(position, square, color).is_empty()
}

pub fn in_check(position: &Position, color: Color) -> bool {
    let Some(wazir_square) = position.wazir_square(color) else {
        return false;
    };
    is_attacked_by(position, wazir_square, color.opposite())
}

pub fn any_pseudomoves<'a>(position: &'a Position) -> impl Iterator<Item = AnyMove> + 'a {
    match position.stage() {
        Stage::Setup => Either::Case0(setup_moves(position.to_move()).map(AnyMove::Setup)),
        Stage::Regular => Either::Case1(pseudomoves(position).map(AnyMove::Regular)),
        Stage::End(_) => panic!("End of game"),
    }
}

/// Generate all pseudomoves.
/// Includes non-escapes and suicides.
pub fn pseudomoves<'a>(position: &'a Position) -> impl Iterator<Item = Move> + 'a {
    pseudocaptures(position)
        .chain(pseudojumps(position))
        .chain(drops(position))
}

/// Generate all moves except suicides.
pub fn moves<'a>(position: &'a Position) -> impl Iterator<Item = Move> + 'a {
    if in_check(position, position.to_move()) {
        Either::Case0(check_evasions(position))
    } else {
        Either::Case1(moves_not_in_check(position))
    }
}

/// Generate all moves.
/// Must not be in check. Does not include suicides.
pub fn moves_not_in_check<'a>(position: &'a Position) -> impl Iterator<Item = Move> + 'a {
    captures(position)
        .chain(jumps(position))
        .chain(drops(position))
}

// Must be in check. Generates all moves that escape the check.
pub fn check_evasions<'a>(position: &'a Position) -> impl Iterator<Item = Move> + 'a {
    check_evasions_capture_attacker(position)
        .chain(captures_by_wazir(position))
        .chain(jumps_by_wazir(position))
}

/// Generate all captures
/// Includes non-escapes and suicides.
pub fn pseudocaptures<'a>(position: &'a Position) -> impl Iterator<Item = Move> + 'a {
    Piece::all().flat_map(move |piece| pseudocaptures_by_piece(position, piece))
}

/// Must not be in check. Generates all captures that are not suicides.
pub fn captures<'a>(position: &'a Position) -> impl Iterator<Item = Move> + 'a {
    Piece::all_non_wazir()
        .flat_map(move |piece| pseudocaptures_by_piece(position, piece))
        .chain(captures_by_wazir(position))
}

/// Must not be in check. Generates all captures that are checks.
pub fn captures_checks<'a>(position: &'a Position) -> impl Iterator<Item = Move> + 'a {
    let me = position.to_move();
    let opp = me.opposite();
    let wazir_square = position.wazir_square(opp).unwrap();
    Piece::all_non_wazir().flat_map(move |piece| {
        let from_mask = double_move_bitboard(piece, wazir_square);
        let to_mask = move_bitboard(piece, wazir_square);
        pseudocaptures_by_piece_masks(position, piece, from_mask, to_mask)
    })
}

/// Must not be in check.
/// Generates all captures that are not checks, and not suicides.
pub fn captures_non_checks<'a>(position: &'a Position) -> impl Iterator<Item = Move> + 'a {
    let wazir_square = position
        .wazir_square(position.to_move().opposite())
        .unwrap();
    Piece::all_non_wazir()
        .flat_map(move |piece| {
            let to_mask = !move_bitboard(piece, wazir_square);
            pseudocaptures_by_piece_masks(position, piece, Bitboard::ALL, to_mask)
        })
        .chain(captures_by_wazir(position))
}

/// Must not be in check. Generates all captures that are check threats.
pub fn captures_check_threats<'a>(position: &'a Position) -> impl Iterator<Item = Move> + 'a {
    let opp = position.to_move().opposite();
    let wazir_square = position.wazir_square(opp).unwrap();
    Piece::all_non_wazir().flat_map(move |piece| {
        let from_mask = triple_move_bitboard(piece, wazir_square);
        let to_mask = double_move_bitboard(piece, wazir_square);
        pseudocaptures_by_piece_masks(position, piece, from_mask, to_mask)
    })
}

/// Must not be in check.
/// Generates all captures that are not checks, not check threats, and not suicides.
pub fn captures_boring<'a>(position: &'a Position) -> impl Iterator<Item = Move> + 'a {
    let wazir_square = position
        .wazir_square(position.to_move().opposite())
        .unwrap();
    Piece::all_non_wazir()
        .flat_map(move |piece| {
            let to_mask =
                !(move_bitboard(piece, wazir_square) | double_move_bitboard(piece, wazir_square));
            pseudocaptures_by_piece_masks(position, piece, Bitboard::ALL, to_mask)
        })
        .chain(captures_by_wazir(position))
}

/// Generates all captures by the wazir.
pub fn captures_by_wazir<'a>(position: &'a Position) -> impl Iterator<Item = Move> + 'a {
    let opp = position.to_move().opposite();
    pseudocaptures_by_piece(position, Piece::Wazir)
        .filter(move |mov| !is_attacked_by(position, mov.to, opp))
}

fn pseudocaptures_by_piece<'a>(
    position: &'a Position,
    piece: Piece,
) -> impl Iterator<Item = Move> + 'a {
    pseudocaptures_by_piece_masks(position, piece, Bitboard::ALL, Bitboard::ALL)
}

fn pseudocaptures_by_piece_masks<'a>(
    position: &'a Position,
    piece: Piece,
    from_mask: Bitboard,
    to_mask: Bitboard,
) -> impl Iterator<Item = Move> + 'a {
    let me = position.to_move();
    let opp = me.opposite();
    let colored_piece = piece.with_color(me);
    let from_mask = from_mask & position.occupied_by_piece(colored_piece);
    let to_mask = to_mask & position.occupied_by(opp);
    from_mask.into_iter().flat_map(move |from| {
        (move_bitboard(piece, from) & to_mask)
            .into_iter()
            .map(move |to| Move {
                colored_piece,
                from: Some(from),
                captured: Some(position.square(to).unwrap().piece()),
                to,
            })
    })
}

// Generates all captures of the wazir, i.e. final moves of the game.
pub fn captures_of_wazir<'a>(position: &'a Position) -> impl Iterator<Item = Move> + 'a {
    let wazir_square = position
        .wazir_square(position.to_move().opposite())
        .unwrap();
    pseudocaptures_of_square(position, wazir_square)
}

// Must be in check.
// Generates all captures that capture the checking piece.
pub fn check_evasions_capture_attacker<'a>(
    position: &'a Position,
) -> impl Iterator<Item = Move> + 'a {
    let me = position.to_move();
    let opp = me.opposite();
    let wazir_square = position.wazir_square(me).unwrap();
    let checked_by = attacked_by(position, wazir_square, opp);
    let mut checked_by_iter = checked_by.into_iter();
    let mut only_checked_by = Some(checked_by_iter.next().expect("Not in check"));
    if checked_by_iter.next().is_some() {
        // checked by multiple pieces
        only_checked_by = None;
    }
    // It's OK to use pseudocaptures here because there is only one attacker.
    // Wazir-wazir capture is fine.
    only_checked_by
        .into_iter()
        .flat_map(move |to| pseudocaptures_of_square(position, to))
}

// Generate all captures of a piece on a square.
fn pseudocaptures_of_square<'a>(
    position: &'a Position,
    to: Square,
) -> impl Iterator<Item = Move> + 'a {
    assert!(position.stage() == Stage::Regular);
    let me = position.to_move();
    let opp = me.opposite();
    let captured = position.square(to).unwrap();
    assert_eq!(captured.color(), opp);
    let captured = captured.piece();
    Piece::all().flat_map(move |piece| {
        let colored_piece = piece.with_color(me);
        let from_bitboard = move_bitboard(piece, to) & position.occupied_by_piece(colored_piece);
        from_bitboard.into_iter().map(move |from| Move {
            colored_piece,
            from: Some(from),
            captured: Some(captured),
            to,
        })
    })
}

/// Generate all pseudojumps (not captures).
/// Includes non-escapes and suicides.
pub fn pseudojumps<'a>(position: &'a Position) -> impl Iterator<Item = Move> + 'a {
    Piece::all().flat_map(move |piece| pseudojumps_by_piece(position, piece))
}

/// Must not be in check. Generates jumps that are not suicides.
pub fn jumps<'a>(position: &'a Position) -> impl Iterator<Item = Move> + 'a {
    Piece::all_non_wazir()
        .flat_map(move |piece| pseudojumps_by_piece(position, piece))
        .chain(jumps_by_wazir(position))
}

/// Must not be in check. Generates all jumps that are checks and not suicides.
pub fn jumps_checks<'a>(position: &'a Position) -> impl Iterator<Item = Move> + 'a {
    let opp = position.to_move().opposite();
    let wazir_square = position.wazir_square(opp).unwrap();
    Piece::all_non_wazir().flat_map(move |piece| {
        let from_mask = double_move_bitboard(piece, wazir_square);
        let to_mask = move_bitboard(piece, wazir_square);
        pseudojumps_by_piece_masks(position, piece, from_mask, to_mask)
    })
}

/// Must not be in check. Generates all jumps that are check threats and not suicides.
pub fn jumps_check_threats<'a>(position: &'a Position) -> impl Iterator<Item = Move> + 'a {
    let opp = position.to_move().opposite();
    let wazir_square = position.wazir_square(opp).unwrap();
    Piece::all_non_wazir().flat_map(move |piece| {
        let from_mask = triple_move_bitboard(piece, wazir_square);
        let to_mask = double_move_bitboard(piece, wazir_square);
        pseudojumps_by_piece_masks(position, piece, from_mask, to_mask)
    })
}

/// Must not be in check.
/// Generates all jumps that are not checks and not check threats and not suicides.
pub fn jumps_boring<'a>(position: &'a Position) -> impl Iterator<Item = Move> + 'a {
    let opp = position.to_move().opposite();
    let wazir_square = position.wazir_square(opp).unwrap();
    Piece::all_non_wazir()
        .flat_map(move |piece| {
            let to_mask =
                !(move_bitboard(piece, wazir_square) | double_move_bitboard(piece, wazir_square));
            pseudojumps_by_piece_masks(position, piece, Bitboard::ALL, to_mask)
        })
        .chain(jumps_by_wazir(position))
}

fn pseudojumps_by_piece<'a>(
    position: &'a Position,
    piece: Piece,
) -> impl Iterator<Item = Move> + 'a {
    pseudojumps_by_piece_masks(position, piece, Bitboard::ALL, Bitboard::ALL)
}

fn pseudojumps_by_piece_masks<'a>(
    position: &'a Position,
    piece: Piece,
    from_mask: Bitboard,
    to_mask: Bitboard,
) -> impl Iterator<Item = Move> + 'a {
    assert!(position.stage() == Stage::Regular);
    let me = position.to_move();
    let colored_piece = piece.with_color(me);
    let from_mask = from_mask & position.occupied_by_piece(colored_piece);
    let to_mask = to_mask & position.empty_squares();
    from_mask.into_iter().flat_map(move |from| {
        (move_bitboard(piece, from) & to_mask)
            .into_iter()
            .map(move |to| Move {
                colored_piece,
                from: Some(from),
                captured: None,
                to,
            })
    })
}

// Generates all Wazir jumps that are not suicides.
pub fn jumps_by_wazir<'a>(position: &'a Position) -> impl Iterator<Item = Move> + 'a {
    let opp = position.to_move().opposite();
    pseudojumps_by_piece(position, Piece::Wazir)
        .filter(move |mov| !is_attacked_by(position, mov.to, opp))
}

/// Piece drops.
/// If in check, these are non-escapes.
pub fn drops<'a>(position: &'a Position) -> impl Iterator<Item = Move> + 'a {
    Piece::all_non_wazir()
        .flat_map(move |piece| drops_piece_to_mask(position, piece, !Bitboard::EMPTY))
}

/// Piece drops that are checks.
/// If in check, these are non-escapes.
pub fn drops_checks<'a>(position: &'a Position) -> impl Iterator<Item = Move> + 'a {
    let wazir_square = position
        .wazir_square(position.to_move().opposite())
        .unwrap();
    Piece::all_non_wazir().flat_map(move |piece| {
        drops_piece_to_mask(position, piece, move_bitboard(piece, wazir_square))
    })
}

/// Piece drops that are checks.
/// If in check, these are non-escapes.
pub fn drops_check_threats<'a>(position: &'a Position) -> impl Iterator<Item = Move> + 'a {
    let wazir_square = position
        .wazir_square(position.to_move().opposite())
        .unwrap();
    Piece::all_non_wazir().flat_map(move |piece| {
        drops_piece_to_mask(position, piece, double_move_bitboard(piece, wazir_square))
    })
}

/// Piece drops that are not checks and not check threats.
/// If in check, these are non-escapes.
pub fn drops_boring<'a>(position: &'a Position) -> impl Iterator<Item = Move> + 'a {
    let wazir_square = position
        .wazir_square(position.to_move().opposite())
        .unwrap();
    Piece::all_non_wazir().flat_map(move |piece| {
        let to_mask =
            !(move_bitboard(piece, wazir_square) | double_move_bitboard(piece, wazir_square));
        drops_piece_to_mask(position, piece, to_mask)
    })
}

/// Piece drops with a to mask.
/// If in check, these are non-escapes.
fn drops_piece_to_mask<'a>(
    position: &'a Position,
    piece: Piece,
    to_mask: Bitboard,
) -> impl Iterator<Item = Move> + 'a {
    assert!(position.stage() == Stage::Regular);
    let me = position.to_move();
    let colored_piece = piece.with_color(me);
    let targets = if position.num_captured(colored_piece) > 0 {
        position.empty_squares() & to_mask
    } else {
        Bitboard::EMPTY
    };
    targets.into_iter().map(move |to| Move {
        colored_piece,
        from: None,
        captured: None,
        to,
    })
}
