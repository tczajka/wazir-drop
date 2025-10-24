use crate::{random_opening, run_game};
use rand::Rng;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use threadpool::ThreadPool;
use wazir_drop::{Color, DEFAULT_TIME_LIMIT, PlayerFactory, enums::EnumMap};

#[derive(Debug, Clone)]
pub struct MatchResult {
    pub num_games: usize,
    pub player0_score: isize,
    pub min_time_left: [Duration; 2],
}

pub fn run_match<RNG: Rng>(
    match_id: &str,
    num_double_games: usize,
    num_threads: usize,
    opening_length: usize,
    player_factories: [Arc<dyn PlayerFactory>; 2],
    time_limits: [Option<Duration>; 2],
    rng: &mut RNG,
) -> MatchResult {
    let thread_pool = ThreadPool::new(num_threads);
    let match_result = Arc::new(Mutex::new(MatchResult {
        num_games: 0,
        player0_score: 0,
        min_time_left: time_limits.map(|limit| limit.unwrap_or(DEFAULT_TIME_LIMIT)),
    }));
    for game_id in 0..num_double_games {
        let opening = random_opening(opening_length, rng);
        for red_player_idx in 0..2 {
            let game_name = format!("{match_id}-{game_id}-{red_player_idx}");
            let opening = opening.clone();
            let player_factories = player_factories.clone();
            let match_result = match_result.clone();
            thread_pool.execute(move || {
                let pf = EnumMap::from_fn(|color: Color| {
                    &*player_factories[red_player_idx ^ color.index()]
                });
                let tl =
                    EnumMap::from_fn(|color: Color| time_limits[red_player_idx ^ color.index()]);
                let finished_game = run_game(&game_name, pf, &opening, tl);

                let red_score = match finished_game.winner {
                    None => 0,
                    Some(Color::Red) => 1,
                    Some(Color::Blue) => -1,
                };
                let player0_score = if red_player_idx == 0 {
                    red_score
                } else {
                    -red_score
                };

                let mut match_result = match_result.lock().unwrap();
                match_result.num_games += 1;
                match_result.player0_score += player0_score;
                for i in 0..2 {
                    match_result.min_time_left[i] = match_result.min_time_left[i]
                        .min(finished_game.time_left[Color::from_index(i ^ red_player_idx)]);
                }
                log::info!("{game_name} score {player0_score}");
            });
        }
    }
    thread_pool.join();
    match_result.lock().unwrap().clone()
}
