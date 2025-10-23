use std::time::Duration;

use rand::{SeedableRng, rngs::StdRng};
use random_player::RandomPlayer;
use wazir_drop::enums::EnumMap;

#[test]
fn test_referee() {
    let mut rng = StdRng::from_os_rng();
    for opening_len in [0, 2] {
        let opening = referee::random_opening(opening_len, &mut rng);
        _ = referee::run_game::<RandomPlayer, RandomPlayer>(
            &opening,
            EnumMap::from_fn(|_| Duration::from_secs(1)),
        );
    }
}
