use std::str::FromStr;
use wazir_drop::{enums::SimpleEnumExt, Color};

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
        assert_eq!(Color::from_str(&name).unwrap(), color);
    }
}

#[test]
fn test_initial_squares() {
    assert_eq!(
        Color::Red.initial_squares().to_string(),
        "\
xxxxxxxx
xxxxxxxx
........
........
........
........
........
........
"
    );

    assert_eq!(
        Color::Blue.initial_squares().to_string(),
        "\
........
........
........
........
........
........
xxxxxxxx
xxxxxxxx
"
    );
}
