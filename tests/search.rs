use std::str::FromStr;

use wazir_drop::{RegularMove, Variation};

#[test]
fn test_variation() {
    let mut variation = Variation::empty();
    variation = variation.add_front(RegularMove::from_str("A@a1").unwrap());
    variation = variation.add_front(RegularMove::from_str("a@a2").unwrap());
    assert_eq!(variation.to_string(), "a@a2 A@a1");
    assert!(!variation.truncated);

    for _ in 0..200 {
        variation = variation.add_front(RegularMove::from_str("A@a1").unwrap());
    }
    assert!(variation.truncated);
    assert_eq!(variation.len(), 102);
}
