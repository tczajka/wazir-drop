use wazir_drop::{parser::ParserExt, Color};

#[test]
fn test_opposite() {
    assert_eq!(Color::Red.opposite(), Color::Blue);
    assert_eq!(Color::Blue.opposite(), Color::Red);
}

#[test]
fn test_parse() {
    assert_eq!(Color::parser().parse_all(b"red"), Ok(Color::Red));
    assert_eq!(Color::parser().parse_all(b"blue"), Ok(Color::Blue));
    assert!(Color::parser().parse_all(b"green").is_err());
}

#[test]
fn test_display() {
    assert_eq!(Color::Red.to_string(), "red");
    assert_eq!(Color::Blue.to_string(), "blue");
}
