use std::str::FromStr;
use wazir_drop::{
    enum_map::SimpleEnumExt, parser::ParseError, ColoredOpeningMove, OpeningMove, Piece,
};

#[test]
fn test_opening_move_size_matches_piece_initial_count() {
    let num_pieces = Piece::all().map(Piece::initial_count).sum();
    assert_eq!(OpeningMove::SIZE, num_pieces);
}

#[test]
fn test_colored_opening_move_display_from_str() {
    let com = ColoredOpeningMove::from_str("AWNAADADAFFAADDA").unwrap();
    assert_eq!(com.to_string(), "AWNAADADAFFAADDA");
    let com = ColoredOpeningMove::from_str("aaaaaffdaddadnwa").unwrap();
    assert_eq!(com.to_string(), "aaaaaffdaddadnwa");

    assert_eq!(
        ColoredOpeningMove::from_str("WWWWWWWWWWWWWWWW"),
        Err(ParseError)
    );
    assert_eq!(
        ColoredOpeningMove::from_str("AWNAADADAFFAADDa"),
        Err(ParseError)
    );
    assert_eq!(ColoredOpeningMove::from_str("W"), Err(ParseError));
}
