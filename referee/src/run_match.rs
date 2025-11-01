use crate::{random_opening, run_game};
use rand::Rng;
use std::{
    fmt::{self, Display, Formatter},
    sync::{Arc, Mutex},
    time::Duration,
};
use threadpool::ThreadPool;
use wazir_drop::{Color, Outcome, PlayerFactory, constants::DEFAULT_TIME_LIMIT, enums::EnumMap};

#[derive(Debug, Clone)]
pub struct MatchResult {
    pub match_id: String,
    pub num_games: usize,
    pub num_draws: usize,
    pub player0_points: i32,
    pub total_game_length: usize,
    pub min_time_left: [Duration; 2],
}

impl Display for MatchResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Match {}: ", self.match_id)?;
        writeln!(f, "  Games: {}", self.num_games)?;
        writeln!(f, "  Score: {}", self.player0_points)?;
        let score_per_game = self.player0_points as f64 / self.num_games as f64;
        let score_per_game_2stddev = 2.0 / (self.num_games as f64).sqrt();
        writeln!(
            f,
            "  Score per game: {score_per_game:.3}  +- {score_per_game_2stddev:.3}",
        )?;
        writeln!(
            f,
            "  Draws: {:.2}%",
            self.num_draws as f64 / self.num_games as f64 * 100.0
        )?;
        writeln!(
            f,
            "  Average game length: {:.2}",
            self.total_game_length as f64 / self.num_games as f64
        )?;
        // win_prob = 1 / (1 + 10^(-elo_diff / 400))
        // win_prob = (score_per_game + 1) / 2
        // 1 / (1 + 10^(-elo_diff / 400)) = (score_per_game + 1) / 2
        // 1 + 10^(-elo_diff / 400) = 2 / (score_per_game + 1)
        // 10^(-elo_diff / 400) = 2 / (score_per_game + 1) - 1
        // -elo_diff / 400 = log10(2 / (score_per_game + 1) - 1)
        // elo_diff = -400 * log10(2 / (score_per_game + 1) - 1)
        let elo_diff = -400.0 * (2.0 / (score_per_game + 1.0) - 1.0).log10();
        writeln!(f, "  ELO: {elo_diff:.3}")?;
        write!(f, "  Min time left:")?;
        for t in self.min_time_left {
            write!(f, " {}", t.as_millis())?;
        }
        writeln!(f)?;
        Ok(())
    }
}

pub fn run_match<RNG: Rng>(
    match_id: &str,
    num_rounds: usize,
    num_threads: usize,
    opening_length: usize,
    player_factories: [Arc<dyn PlayerFactory>; 2],
    time_limits: [Option<Duration>; 2],
    rng: &mut RNG,
) -> MatchResult {
    let thread_pool = ThreadPool::new(num_threads);
    let match_result = Arc::new(Mutex::new(MatchResult {
        match_id: match_id.to_string(),
        num_games: 0,
        num_draws: 0,
        player0_points: 0,
        total_game_length: 0,
        min_time_left: time_limits.map(|limit| limit.unwrap_or(DEFAULT_TIME_LIMIT)),
    }));
    for round in 0..num_rounds {
        let opening = random_opening(opening_length, rng);
        for red_player_idx in 0..2 {
            let game_id = format!("{match_id}-{round}-{red_player_idx}");
            let opening = opening.clone();
            let player_factories = player_factories.clone();
            let match_result = match_result.clone();
            thread_pool.execute(move || {
                let pf = EnumMap::from_fn(|color: Color| {
                    &*player_factories[red_player_idx ^ color.index()]
                });
                let tl =
                    EnumMap::from_fn(|color: Color| time_limits[red_player_idx ^ color.index()]);
                let finished_game = run_game(&game_id, pf, &opening, tl);

                let player0_points = finished_game
                    .outcome
                    .points(Color::from_index(red_player_idx));

                let mut match_result = match_result.lock().unwrap();
                match_result.num_games += 1;
                if finished_game.outcome == Outcome::Draw {
                    match_result.num_draws += 1;
                }
                match_result.total_game_length += finished_game.moves.len();
                match_result.player0_points += player0_points;
                for i in 0..2 {
                    match_result.min_time_left[i] = match_result.min_time_left[i]
                        .min(finished_game.time_left[Color::from_index(i ^ red_player_idx)]);
                }
                log::info!(
                    "{game_id} points {player0_points} total {running_points}",
                    running_points = match_result.player0_points
                );
            });
        }
    }
    thread_pool.join();
    match_result.lock().unwrap().clone()
}
