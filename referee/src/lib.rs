use std::time::Duration;

use rand::rngs::StdRng;
use wazir_drop::{
    Color, DEFAULT_TIME_LIMIT, MAX_MOVES_IN_GAME, Move, Player, Position, Stage, clock::Timer,
    enums::EnumMap, movegen,
};

pub fn random_opening(len: usize, rng: &mut StdRng) -> Vec<Move> {
    let mut moves = Vec::new();
    let mut position = Position::initial();
    while moves.len() < len && position.stage() != Stage::End {
        let mov = movegen::random_move(&position, rng);
        position = position.make_move(mov).unwrap();
        moves.push(mov);
    }
    moves
}

#[derive(Debug, Clone)]
pub struct FinishedGame {
    pub moves: Vec<Move>,
    pub winner: Option<Color>,
    pub time_left: EnumMap<Color, Duration>,
}

pub fn run_game(
    mut players: EnumMap<Color, Box<dyn Player>>,
    opening: &[Move],
    time_limit: EnumMap<Color, Option<Duration>>,
) -> FinishedGame {
    let mut position = Position::initial();
    let mut moves = opening.to_vec();
    let mut winner = None;

    let mut timers =
        EnumMap::from_fn(|color| Timer::new(time_limit[color].unwrap_or(DEFAULT_TIME_LIMIT)));

    for (color, player) in players.iter_mut() {
        timers[color].start();
        player.init(color, opening, time_limit[color]);
        timers[color].stop();
    }
    for &mov in opening {
        position = position.make_move(mov).expect("Invalid opening move");
    }

    loop {
        let color = position.to_move();
        let opp = color.opposite();

        if position.stage() == Stage::End {
            winner = Some(opp);
            break;
        }
        if moves.len() >= MAX_MOVES_IN_GAME {
            break;
        }
        timers[color].start();
        let mov = players[color].make_move(&position, &timers[color]);
        timers[color].stop();

        timers[opp].start();
        players[opp].opponent_move(&position, mov, &timers[opp]);
        timers[opp].stop();

        moves.push(mov);
        position = position.make_move(mov).expect("Invalid move");
    }

    FinishedGame {
        moves,
        winner,
        time_left: EnumMap::from_fn(|color| timers[color].get()),
    }
}
