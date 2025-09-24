use wazir_drop::{enum_map::SimpleEnumExt, parser::ParserExt, Color};

#[test]
fn test_opposite() {
    for color in Color::all() {
        assert_eq!(color.opposite().opposite(), color);
    }
}

#[test]
fn test_display_round_trip() {
    for color in Color::all() {
        let name = color.to_string();
        assert_eq!(Color::parser().parse_all(name.as_bytes()), Ok(color));
    }
}
