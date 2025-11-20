use crate::{ExtendableVariation, LongVariation, Move, PVTable, Variation};
use std::str::FromStr;

#[test]
fn test_ttable() {
    let mut ttable = PVTable::new(1 << 14);
    let hash = 0x1234567890abcdef;
    let variation = LongVariation::empty().add_front(Move::from_str("A@a1").unwrap());
    ttable.set(hash, variation.clone());
    assert_eq!(ttable.get(hash).unwrap().to_string(), "A@a1");
    assert!(ttable.get(hash + 1).is_none());
}
