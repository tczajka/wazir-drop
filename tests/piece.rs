use wazir_drop::{
    enum_map::SimpleEnumExt, parser::ParserExt, Color, ColoredPiece, Piece, PieceNonWazir,
};

#[test]
fn test_piece_non_wazir() {
    for piece in PieceNonWazir::all() {
        assert_eq!(PieceNonWazir::try_from(Piece::from(piece)), Ok(piece));
    }
    assert!(PieceNonWazir::try_from(Piece::Wazir).is_err());
}

#[test]
fn test_display_round_trip() {
    for color in Color::all() {
        for piece in Piece::all() {
            let colored_piece = ColoredPiece { color, piece };
            let name = colored_piece.to_string();
            assert_eq!(
                ColoredPiece::parser().parse_all(name.as_bytes()),
                Ok(ColoredPiece { color, piece })
            );
        }
    }
}
