use std::str::FromStr;
use wazir_drop::{
    constants::MAX_VARIATION_LENGTH, ExtendableVariation, LongVariation, RegularMove, Variation,
};

#[test]
fn test_long_variation() {
    let mut variation = LongVariation::empty();
    variation = variation.add_front(RegularMove::from_str("A@a1").unwrap());
    variation = variation.add_front(RegularMove::from_str("a@a2").unwrap());
    assert_eq!(variation.to_string(), "a@a2 A@a1");
    assert!(!variation.truncated);

    for _ in 0..MAX_VARIATION_LENGTH + 3 {
        variation = variation.add_front(RegularMove::from_str("A@a1").unwrap());
    }
    assert!(variation.truncated);
    assert_eq!(variation.len(), MAX_VARIATION_LENGTH);
}
