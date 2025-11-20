use std::time::Duration;
use wazir_drop::{
    AnyMove, Color, Outcome, PlayerFactory, Position, Stage, clock::Timer,
    constants::DEFAULT_TIME_LIMIT, enums::EnumMap,
};

#[derive(Debug, Clone)]
pub struct FinishedGame {
    pub moves: Vec<AnyMove>,
    pub outcome: Outcome,
    pub time_left: EnumMap<Color, Duration>,
}

pub fn run_game(
    game_id: &str,
    player_factories: EnumMap<Color, &dyn PlayerFactory>,
    opening: &[AnyMove],
    time_limit: EnumMap<Color, Option<Duration>>,
) -> FinishedGame {
    let mut position = Position::initial();
    let mut moves = opening.to_vec();

    let mut timers =
        EnumMap::from_fn(|color| Timer::new(time_limit[color].unwrap_or(DEFAULT_TIME_LIMIT)));

    let mut players = EnumMap::from_fn(|color| {
        timers[color].start();
        let player = player_factories[color].create(game_id, color, opening, time_limit[color]);
        timers[color].stop();
        player
    });

    for &mov in opening {
        position = position.make_any_move(mov).expect("Invalid opening move");
    }

    let outcome = loop {
        let color = position.to_move();
        let opp = color.opposite();

        if let Stage::End(outcome) = position.stage() {
            break outcome;
        }
        timers[color].start();
        let mov = players[color].make_move(&position, &timers[color]);
        timers[color].stop();

        moves.push(mov);
        let new_position = position.make_any_move(mov).expect("Invalid move");

        if !matches!(new_position.stage(), Stage::End(_)) {
            timers[opp].start();
            players[opp].opponent_move(&position, mov, &timers[opp]);
            timers[opp].stop();
        }

        position = new_position;
    };

    FinishedGame {
        moves,
        outcome,
        time_left: EnumMap::from_fn(|color| timers[color].get()),
    }
}
