use crate::{
    either::Either,
    enums::{EnumMap, SimpleEnumExt},
    smallvec::SmallVec,
    Bitboard, Color, InvalidMove, Move, Piece, Position, RegularMove, SetupMove, ShortMove,
    ShortMoveFrom, Square, Stage,
};
use std::iter;

pub fn move_bitboard(piece: Piece, square: Square) -> Bitboard {
    MOVE_BITBOARD_TABLE[piece][square]
}

pub fn validate_from_to(piece: Piece, from: Square, to: Square) -> Result<(), InvalidMove> {
    if !move_bitboard(piece, from).contains(to) {
        return Err(InvalidMove);
    }
    Ok(())
}

static MOVE_BITBOARD_TABLE: EnumMap<Piece, EnumMap<Square, Bitboard>> = {
    let mut table = [EnumMap::from_array([Bitboard::EMPTY; Square::COUNT]); Piece::COUNT];
    let mut piece_idx = 0;
    while piece_idx != Piece::COUNT {
        table[piece_idx] = calc_move_bitboard_table_for_piece(Piece::from_index(piece_idx));
        piece_idx += 1;
    }
    EnumMap::from_array(table)
};

const fn calc_move_bitboard_table_for_piece(piece: Piece) -> EnumMap<Square, Bitboard> {
    let mut table = [Bitboard::EMPTY; Square::COUNT];
    let mut square_idx = 0;
    while square_idx != Square::COUNT {
        table[square_idx] = calc_move_bitboard(piece, Square::from_index(square_idx));
        square_idx += 1;
    }
    EnumMap::from_array(table)
}

const fn calc_move_bitboard(piece: Piece, square: Square) -> Bitboard {
    let mut bitboard = Bitboard::EMPTY;
    let directions = piece.directions();
    let mut i = 0;
    while i != directions.len() {
        if let Some(square2) = square.add(directions[i]) {
            bitboard = bitboard.with_square(square2);
        }
        i += 1;
    }
    bitboard
}

pub fn move_from_short_move(
    position: &Position,
    short_move: ShortMove,
) -> Result<Move, InvalidMove> {
    match short_move {
        ShortMove::Setup(mov) => {
            if position.stage() != Stage::Setup || mov.color != position.to_move() {
                return Err(InvalidMove);
            }
            mov.validate_pieces()?;
            Ok(Move::Setup(mov))
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
            Ok(Move::Regular(RegularMove {
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

pub fn pseudomoves<'a>(position: &'a Position) -> impl Iterator<Item = Move> + 'a {
    match position.stage() {
        Stage::Setup => Either::Left(setup_moves(position.to_move()).map(Move::Setup)),
        Stage::Regular => Either::Right(regular_pseudomoves(position).map(Move::Regular)),
        Stage::End(_) => panic!("End of game"),
    }
}

/// Generate all regular pseudomoves.
/// Includes non-escapes and suicides.
pub fn regular_pseudomoves<'a>(position: &'a Position) -> impl Iterator<Item = RegularMove> + 'a {
    pseudocaptures(position)
        .chain(pseudojumps(position))
        .chain(drops(position))
}

/// Generate all moves except suicides.
pub fn regular_moves<'a>(position: &'a Position) -> impl Iterator<Item = RegularMove> + 'a {
    if in_check(position, position.to_move()) {
        Either::Left(check_evasions(position))
    } else {
        Either::Right(regular_moves_not_in_check(position))
    }
}

/// Generate all moves.
/// Must not be in check. Does not include suicides.
pub fn regular_moves_not_in_check<'a>(
    position: &'a Position,
) -> impl Iterator<Item = RegularMove> + 'a {
    captures(position)
        .chain(jumps(position))
        .chain(drops(position))
}

/// Generate all captures
/// Includes non-escapes and suicides.
pub fn pseudocaptures<'a>(position: &'a Position) -> impl Iterator<Item = RegularMove> + 'a {
    Piece::all().flat_map(move |piece| pseudocaptures_by_piece(position, piece))
}

fn pseudocaptures_by_piece<'a>(
    position: &'a Position,
    piece: Piece,
) -> impl Iterator<Item = RegularMove> + 'a {
    assert!(position.stage() == Stage::Regular);
    let me = position.to_move();
    let opp = me.opposite();
    let opp_mask = position.occupied_by(opp);
    let colored_piece = piece.with_color(me);
    position
        .occupied_by_piece(colored_piece)
        .into_iter()
        .flat_map(move |from| {
            (move_bitboard(piece, from) & opp_mask)
                .into_iter()
                .map(move |to| RegularMove {
                    colored_piece,
                    from: Some(from),
                    captured: Some(position.square(to).unwrap().piece()),
                    to,
                })
        })
}

/// Generate all pseudojumps (not captures).
/// Includes non-escapes and suicides.
pub fn pseudojumps<'a>(position: &'a Position) -> impl Iterator<Item = RegularMove> + 'a {
    Piece::all().flat_map(move |piece| pseudojumps_by_piece(position, piece))
}

fn pseudojumps_by_piece<'a>(
    position: &'a Position,
    piece: Piece,
) -> impl Iterator<Item = RegularMove> + 'a {
    assert!(position.stage() == Stage::Regular);
    let me = position.to_move();
    let empty = position.empty_squares();
    let colored_piece = piece.with_color(me);
    position
        .occupied_by_piece(colored_piece)
        .into_iter()
        .flat_map(move |from| {
            (move_bitboard(piece, from) & empty)
                .into_iter()
                .map(move |to| RegularMove {
                    colored_piece,
                    from: Some(from),
                    captured: None,
                    to,
                })
        })
}

/// Piece drops.
/// If in check, these are non-escapes.
pub fn drops<'a>(position: &'a Position) -> impl Iterator<Item = RegularMove> + 'a {
    assert!(position.stage() == Stage::Regular);
    let me = position.to_move();
    let empty = position.empty_squares();
    Piece::all()
        .map(move |piece| piece.with_color(me))
        .filter(move |&cpiece| position.num_captured(cpiece) > 0)
        .flat_map(move |colored_piece| {
            empty.into_iter().map(move |to| RegularMove {
                colored_piece,
                from: None,
                captured: None,
                to,
            })
        })
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
    let Some(wazir_square) = position
        .occupied_by_piece(Piece::Wazir.with_color(color))
        .first()
    else {
        return false;
    };
    is_attacked_by(position, wazir_square, color.opposite())
}

// Generates all captures of the wazir, i.e. final moves of the game.
pub fn captures_of_wazir<'a>(position: &'a Position) -> impl Iterator<Item = RegularMove> + 'a {
    assert!(position.stage() == Stage::Regular);
    let me = position.to_move();
    let opp = me.opposite();
    let wazir_square = position
        .occupied_by_piece(Piece::Wazir.with_color(opp))
        .first()
        .unwrap();
    pseudocaptures_of_square(position, wazir_square)
}

// Generate all captures of a piece on a square.
fn pseudocaptures_of_square<'a>(
    position: &'a Position,
    to: Square,
) -> impl Iterator<Item = RegularMove> + 'a {
    assert!(position.stage() == Stage::Regular);
    let me = position.to_move();
    let opp = me.opposite();
    let captured = position.square(to).unwrap();
    assert_eq!(captured.color(), opp);
    let captured = captured.piece();
    Piece::all().flat_map(move |piece| {
        let colored_piece = piece.with_color(me);
        let from_bitboard = move_bitboard(piece, to) & position.occupied_by_piece(colored_piece);
        from_bitboard.into_iter().map(move |from| RegularMove {
            colored_piece,
            from: Some(from),
            captured: Some(captured),
            to,
        })
    })
}

// Must be in check. Generates all moves that escape the check.
pub fn check_evasions<'a>(position: &'a Position) -> impl Iterator<Item = RegularMove> + 'a {
    check_evasions_capture_attacker(position)
        .chain(captures_by_wazir(position))
        .chain(jumps_by_wazir(position))
}

// Must be in check.
// Generates all captures that capture the checking piece.
pub fn check_evasions_capture_attacker<'a>(
    position: &'a Position,
) -> impl Iterator<Item = RegularMove> + 'a {
    assert!(position.stage() == Stage::Regular);
    let me = position.to_move();
    let opp = me.opposite();
    let wazir_square = position
        .occupied_by_piece(Piece::Wazir.with_color(me))
        .first()
        .unwrap();
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

// Must not be in check. Generates all captures that are not suicides.
pub fn captures<'a>(position: &'a Position) -> impl Iterator<Item = RegularMove> + 'a {
    Piece::all_non_wazir()
        .flat_map(move |piece| pseudocaptures_by_piece(position, piece))
        .chain(captures_by_wazir(position))
}

pub fn captures_by_wazir<'a>(position: &'a Position) -> impl Iterator<Item = RegularMove> + 'a {
    let opp = position.to_move().opposite();
    pseudocaptures_by_piece(position, Piece::Wazir)
        .filter(move |mov| !is_attacked_by(position, mov.to, opp))
}

// Must not be in check.
// Generates jumps that are not suicides.
pub fn jumps<'a>(position: &'a Position) -> impl Iterator<Item = RegularMove> + 'a {
    Piece::all_non_wazir()
        .flat_map(move |piece| pseudojumps_by_piece(position, piece))
        .chain(jumps_by_wazir(position))
}

// Generates all Wazir jumps that are not suicides.
pub fn jumps_by_wazir<'a>(position: &'a Position) -> impl Iterator<Item = RegularMove> + 'a {
    let opp = position.to_move().opposite();
    pseudojumps_by_piece(position, Piece::Wazir)
        .filter(move |mov| !is_attacked_by(position, mov.to, opp))
}
