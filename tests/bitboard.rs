use wazir_drop::{Bitboard, Square};

#[test]
fn test_empty() {
    assert_eq!(
        Bitboard::EMPTY.to_string(),
        "\
........
........
........
........
........
........
........
........
"
    );
}

#[test]
fn test_is_empty() {
    assert!(Bitboard::EMPTY.is_empty());
    assert!(!Bitboard::single(Square::B3).is_empty());
}

#[test]
fn test_single() {
    let bitboard = Bitboard::single(Square::B3);
    assert_eq!(
        bitboard.to_string(),
        "\
........
..x.....
........
........
........
........
........
........
"
    );
}

#[test]
fn test_contains() {
    let bitboard = Bitboard::single(Square::B3);
    assert!(bitboard.contains(Square::B3));
    assert!(!bitboard.contains(Square::B4));
}

#[test]
fn test_add() {
    let mut bitboard = Bitboard::EMPTY;
    bitboard.add(Square::B3);
    assert!(bitboard.contains(Square::B3));
}

#[test]
fn test_remove() {
    let mut bitboard = Bitboard::single(Square::B3) | Bitboard::single(Square::B4);
    bitboard.remove(Square::B3);
    assert!(!bitboard.contains(Square::B3));
    assert!(bitboard.contains(Square::B4));
}

#[test]
fn test_first() {
    let bitboard = Bitboard::single(Square::B3) | Bitboard::single(Square::B4);
    assert_eq!(bitboard.first(), Some(Square::B3));
    let bitboard = Bitboard::EMPTY;
    assert_eq!(bitboard.first(), None);
}

#[test]
fn test_or() {
    let a = Bitboard::single(Square::B3);
    let b = Bitboard::single(Square::B4);
    let c = a | b;
    assert_eq!(
        c.to_string(),
        "\
........
..xx....
........
........
........
........
........
........
"
    );

    let mut d = a;
    d |= b;
    assert_eq!(d, a | b);
}

#[test]
fn test_and() {
    let a = Bitboard::single(Square::B3) | Bitboard::single(Square::B4);
    let b = Bitboard::single(Square::B4) | Bitboard::single(Square::B5);
    let c = a & b;
    assert_eq!(c, Bitboard::single(Square::B4));
    let mut d = a;
    d &= b;
    assert_eq!(d, c);
}

#[test]
fn test_xor() {
    let a = Bitboard::single(Square::B3) | Bitboard::single(Square::B4);
    let b = Bitboard::single(Square::B4) | Bitboard::single(Square::B5);
    let c = a ^ b;
    assert_eq!(
        c,
        Bitboard::single(Square::B3) | Bitboard::single(Square::B5)
    );
    let mut d = a;
    d ^= b;
    assert_eq!(d, c);
}

#[test]
fn test_not() {
    let a = Bitboard::single(Square::B3) | Bitboard::single(Square::B4);
    let b = !a;
    assert_eq!(
        b.to_string(),
        "\
xxxxxxxx
xx..xxxx
xxxxxxxx
xxxxxxxx
xxxxxxxx
xxxxxxxx
xxxxxxxx
xxxxxxxx
"
    );
}

#[test]
fn test_count() {
    let a = Bitboard::single(Square::A1) | Bitboard::single(Square::H8);
    assert_eq!(a.count(), 2);
}

#[test]
fn test_is_subset_of() {
    let a = Bitboard::single(Square::B3);
    let b = Bitboard::single(Square::H8);
    assert!(a.is_subset_of(a));
    assert!(a.is_subset_of(a | b));
    assert!(!(a | b).is_subset_of(a));
}

#[test]
fn test_into_iter() {
    let bitboard = Bitboard::single(Square::B3) | Bitboard::single(Square::B4);
    let mut iter = bitboard.into_iter();
    assert_eq!(iter.next(), Some(Square::B3));
    assert_eq!(iter.next(), Some(Square::B4));
    assert_eq!(iter.next(), None);
}
