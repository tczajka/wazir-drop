use std::str::FromStr;
use wazir_drop::{enums::SimpleEnumExt, ColoredPiece, Piece};

#[test]
fn test_all_non_wazir() {
    let pieces: Vec<Piece> = Piece::all_non_wazir().collect();
    assert_eq!(
        pieces,
        vec![Piece::Alfil, Piece::Dabbaba, Piece::Ferz, Piece::Knight]
    );
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
