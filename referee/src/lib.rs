use std::time::Duration;

use rand::rngs::StdRng;
use wazir_drop::{
    Color, MAX_MOVES_IN_GAME, Move, Player, Position, Stage, clock::Timer, enums::EnumMap, movegen,
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

pub fn run_game<PlayerRed: Player, PlayerBlue: Player>(
    opening: &[Move],
    time_limit: EnumMap<Color, Duration>,
) -> FinishedGame {
    let mut position = Position::initial();
    let mut moves = opening.to_vec();
    let mut winner = None;

    for &mov in opening {
        position = position.make_move(mov).expect("Invalid opening move");
    }
    let mut timers = EnumMap::from_fn(|color| Timer::new(time_limit[color]));

    let mut players: EnumMap<Color, Box<dyn Player>> = EnumMap::from_fn(|color| {
        timers[color].start();
        let player: Box<dyn Player> = match color {
            Color::Red => Box::new(PlayerRed::new(color, opening)),
            Color::Blue => Box::new(PlayerBlue::new(color, opening)),
        };
        timers[color].stop();
        player
    });

    loop {
        let color = position.to_move();
        if position.stage() == Stage::End {
            winner = Some(color.opposite());
            break;
        }
        if moves.len() >= MAX_MOVES_IN_GAME {
            break;
        }
        timers[color].start();
        let mov = players[color].make_move(&position, &timers[color]);
        timers[color].stop();

        timers[color.opposite()].start();
        players[color.opposite()].opponent_move(&position, mov);
        timers[color.opposite()].stop();

        moves.push(mov);
        position = position.make_move(mov).expect("Invalid move");
    }

    FinishedGame {
        moves,
        winner,
        time_left: EnumMap::from_fn(|color| timers[color].get()),
    }
}
