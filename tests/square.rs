use std::str::FromStr;
use wazir_drop::{parser::ParseError, Direction, Square};

#[test]
fn test_display() {
    assert_eq!(Square::A1.to_string(), "a1");
    assert_eq!(Square::A8.to_string(), "a8");
    assert_eq!(Square::H1.to_string(), "h1");
    assert_eq!(Square::H8.to_string(), "h8");
    assert_eq!(Square::C5.to_string(), "c5");
}

#[test]
fn test_from_str() {
    assert_eq!(Square::from_str("a1"), Ok(Square::A1));
    assert_eq!(Square::from_str("a8"), Ok(Square::A8));
    assert_eq!(Square::from_str("h1"), Ok(Square::H1));
    assert_eq!(Square::from_str("h8"), Ok(Square::H8));
    assert_eq!(Square::from_str("c5"), Ok(Square::C5));
    assert_eq!(Square::from_str("c9"), Err(ParseError));
    assert_eq!(Square::from_str("i1"), Err(ParseError));
    assert_eq!(Square::from_str("a0"), Err(ParseError));
    assert_eq!(Square::from_str("a9"), Err(ParseError));
    assert_eq!(Square::from_str("ab"), Err(ParseError));
    assert_eq!(Square::from_str("a10"), Err(ParseError));
}

#[test]
fn test_add() {
    assert_eq!(Square::A5.add(Direction::new(-1, 2)), Some(Square::C4));
    assert!(Square::A5.add(Direction::new(-1, -1)).is_none());
    assert!(Square::H5.add(Direction::new(-1, 2)).is_none());
}
