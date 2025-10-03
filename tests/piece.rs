use std::str::FromStr;
use wazir_drop::{enum_map::SimpleEnumExt, ColoredPiece, Piece, PieceNonWazir};

#[test]
fn test_piece_non_wazir() {
    for piece in PieceNonWazir::all() {
        assert_eq!(PieceNonWazir::try_from(Piece::from(piece)), Ok(piece));
    }
    assert!(PieceNonWazir::try_from(Piece::Wazir).is_err());
}

#[test]
fn test_colored_piece_display_round_trip() {
    for cpiece in ColoredPiece::all() {
        let name = cpiece.to_string();
        assert_eq!(ColoredPiece::from_str(&name), Ok(cpiece));
    }
}

#[test]
fn test_colored_piece_parts() {
    for cpiece in ColoredPiece::all() {
        assert_eq!(cpiece, cpiece.piece().with_color(cpiece.color()));
    }
}
