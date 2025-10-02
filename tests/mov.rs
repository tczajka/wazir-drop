use std::str::FromStr;
use wazir_drop::{enum_map::SimpleEnumExt, parser::ParseError, OpeningMove, Piece};

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
