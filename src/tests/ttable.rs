use crate::{
    ttable::{TTable, TTableEntry, TTableScoreType},
    RegularMove, ScoreExpanded,
};
use std::str::FromStr;

#[test]
fn test_ttable() {
    let mut ttable = TTable::new(1024);
    let hash = 0x1234567890abcdef;
    let entry = TTableEntry {
        depth: 10,
        mov: Some(RegularMove::from_str("A@a1").unwrap()),
        score_type: TTableScoreType::Exact,
        score: ScoreExpanded::Eval(100).into(),
    };
    ttable.set(hash, entry);
    assert_eq!(ttable.get(hash), Some(entry));
    assert!(ttable.get(hash + 1).is_none());
}
