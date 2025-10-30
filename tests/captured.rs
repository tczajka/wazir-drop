use std::str::FromStr;
use wazir_drop::{Captured, ColoredPiece};

#[test]
fn test_display_from_str() {
    let captured = Captured::from_str("AWDddd").unwrap();
    assert_eq!(captured.to_string(), "ADWddd");

    assert!(Captured::from_str("FFFFF").is_err());
}

#[test]
fn test_add_remove() {
    let mut captured = Captured::from_str("AWDddd").unwrap();

    captured.add(ColoredPiece::RedAlfil).unwrap();
    captured.remove(ColoredPiece::BlueDabbaba).unwrap();
    assert_eq!(captured.to_string(), "AADWdd");

    assert!(captured.remove(ColoredPiece::BlueAlfil).is_err());
    captured.add(ColoredPiece::RedWazir).unwrap();
    assert!(captured.add(ColoredPiece::RedWazir).is_err());
}

#[test]
fn test_hash() {
    let mut captured = Captured::from_str("AWDddd").unwrap();
    let hash = captured.hash();
    captured.add(ColoredPiece::RedAlfil).unwrap();
    assert_ne!(captured.hash(), hash);
    captured.remove(ColoredPiece::RedAlfil).unwrap();
    assert_eq!(captured.hash(), hash);
}
