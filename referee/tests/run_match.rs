use rand::{SeedableRng, rngs::StdRng};
use random_player::RandomPlayerFactory;
use referee::run_match;
use std::{array, sync::Arc};
use wazir_drop::PlayerFactory;

#[test]
fn test_run_match() {
    let mut rng = StdRng::from_os_rng();
    let player_factories =
        array::from_fn(|_| -> Arc<dyn PlayerFactory> { Arc::new(RandomPlayerFactory::new()) });
    let time_limits = array::from_fn(|_| None);

    let match_results = run_match("test", 10, 2, 2, player_factories, time_limits, &mut rng);

    assert_eq!(match_results.num_games, 20);
}
