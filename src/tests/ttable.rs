use crate::{
    ttable::{TTable, TTableEntry, TTableScore},
    RegularMove, Score,
};
use std::str::FromStr;

#[test]
fn test_ttable() {
    let mut ttable = TTable::new(1024);
    let hash = 0x1234567890abcdef;
    let entry = TTableEntry {
        depth: 10,
        mov: Some(RegularMove::from_str("A@a1").unwrap()),
        score: TTableScore::Exact(Score::from_eval(100)),
    };
    ttable.set(hash, entry);
    assert_eq!(ttable.get(hash), Some(entry));
    assert!(ttable.get(hash + 1).is_none());
}
