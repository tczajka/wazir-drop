use std::str::FromStr;
use wazir_drop::{enums::SimpleEnumExt, parser::ParseError, OpeningMove, Piece, RegularMove};

#[test]
fn test_opening_move_size_matches_piece_initial_count() {
    let num_pieces = Piece::all().map(Piece::initial_count).sum();
    assert_eq!(OpeningMove::SIZE, num_pieces);
}

#[test]
fn test_opening_move_display_from_str() {
    let mov = OpeningMove::from_str("AWNAADADAFFAADDA").unwrap();
    assert_eq!(mov.to_string(), "AWNAADADAFFAADDA");
    let mov = OpeningMove::from_str("aaaaaffdaddadnwa").unwrap();
    assert_eq!(mov.to_string(), "aaaaaffdaddadnwa");

    assert_eq!(OpeningMove::from_str("W"), Err(ParseError));
    assert_eq!(OpeningMove::from_str("AWNAADADAFFAADDa"), Err(ParseError));
}

#[test]
fn test_regular_move_display_from_str() {
    let mov = RegularMove::from_str("A@a1").unwrap();
    assert_eq!(mov.to_string(), "A@a1");
    let mov = RegularMove::from_str("Da1-a3").unwrap();
    assert_eq!(mov.to_string(), "Da1-a3");
    let mov = RegularMove::from_str("Da1xna3").unwrap();
    assert_eq!(mov.to_string(), "Da1xna3");

    assert_eq!(RegularMove::from_str("Aa1"), Err(ParseError));
    assert_eq!(RegularMove::from_str("Da1xNa3"), Err(ParseError));
}
