use std::time::Duration;

use rand::{SeedableRng, rngs::StdRng};
use random_player::RandomPlayer;
use wazir_drop::{Player, enums::EnumMap};

#[test]
fn test_referee() {
    let mut rng = StdRng::from_os_rng();
    for opening_len in [0, 2] {
        let opening = referee::random_opening(opening_len, &mut rng);

        let players = EnumMap::from_fn(|_| {
            let player: Box<dyn Player> = Box::new(RandomPlayer::new());
            player
        });

        _ = referee::run_game(
            players,
            &opening,
            EnumMap::from_fn(|_| Duration::from_secs(1)),
        );
    }
}
