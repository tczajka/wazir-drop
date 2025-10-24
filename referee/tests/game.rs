use rand::{SeedableRng, rngs::StdRng};
use random_player::RandomPlayerFactory;
use wazir_drop::{PlayerFactory, enums::EnumMap};

#[test]
fn test_referee() {
    let mut rng = StdRng::from_os_rng();
    let player_factory = RandomPlayerFactory::new();
    let player_factories = EnumMap::from_fn(|_| &player_factory as &dyn PlayerFactory);
    let time_limits = EnumMap::from_fn(|_| None);

    for opening_len in [0, 2] {
        let opening = referee::random_opening(opening_len, &mut rng);
        _ = referee::run_game("", player_factories, &opening, time_limits);
    }
}
