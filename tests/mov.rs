use std::str::FromStr;
use wazir_drop::{enums::SimpleEnumExt, Move, Piece, RegularMove, SetupMove, ShortMove};

#[test]
fn test_opening_move_size_matches_piece_initial_count() {
    let num_pieces = Piece::all().map(Piece::initial_count).sum();
    assert_eq!(SetupMove::SIZE, num_pieces);
}

#[test]
fn test_opening_move_display_from_str() {
    let mov = SetupMove::from_str("AWNAADADAFFAADDA").unwrap();
    assert_eq!(mov.to_string(), "AWNAADADAFFAADDA");
    let mov = SetupMove::from_str("aaaaaffdaddadnwa").unwrap();
    assert_eq!(mov.to_string(), "aaaaaffdaddadnwa");

    assert!(SetupMove::from_str("W").is_err());
    assert!(SetupMove::from_str("AWNAADADAFFAADDa").is_err());
}

#[test]
fn test_regular_move_display_from_str() {
    let mov = RegularMove::from_str("A@a1").unwrap();
    assert_eq!(mov.to_string(), "A@a1");
    let mov = RegularMove::from_str("Da1-a3").unwrap();
    assert_eq!(mov.to_string(), "Da1-a3");
    let mov = RegularMove::from_str("Da1xna3").unwrap();
    assert_eq!(mov.to_string(), "Da1xna3");

    assert!(RegularMove::from_str("Aa1").is_err());
    assert!(RegularMove::from_str("Da1xNa3").is_err());
}

#[test]
fn test_move_to_short_move() {
    let mov = Move::from_str("AWNAADADAFFAADDA").unwrap();
    assert_eq!(ShortMove::from(mov).to_string(), "AWNAADADAFFAADDA");
    let mov = Move::from_str("d@a3").unwrap();
    assert_eq!(ShortMove::from(mov).to_string(), "da3");
    let mov = Move::from_str("Da1-a3").unwrap();
    assert_eq!(ShortMove::from(mov).to_string(), "a1a3");
    let mov = Move::from_str("Da1xna3").unwrap();
    assert_eq!(ShortMove::from(mov).to_string(), "a1a3");
}

#[test]
fn test_short_move_display_from_str() {
    let mov = ShortMove::from_str("AWNAADADAFFAADDA").unwrap();
    assert_eq!(mov.to_string(), "AWNAADADAFFAADDA");
    let mov = ShortMove::from_str("da3").unwrap();
    assert_eq!(mov.to_string(), "da3");
    let mov = ShortMove::from_str("Na3").unwrap();
    assert_eq!(mov.to_string(), "Na3");
    let mov = ShortMove::from_str("a1a3").unwrap();
    assert_eq!(mov.to_string(), "a1a3");
    assert!(ShortMove::from_str("Da1-a3").is_err());
}

#[test]
fn test_opening_move_validate_pieces() {
    let mov = SetupMove::from_str("AWNAADADAFFAADDA").unwrap();
    assert!(mov.validate_pieces().is_ok());
    let mov = SetupMove::from_str("AWNAADADAFFAADDN").unwrap();
    assert!(mov.validate_pieces().is_err());
}
