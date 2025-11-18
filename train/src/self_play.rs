use extra::{PSFeatures, moverand};
use rand::{SeedableRng, rngs::StdRng, seq::IndexedRandom};
use serde::{Deserialize, Serialize};
use serde_cbor::ser::{IoWrite, Serializer};
use std::{
    error::Error,
    fs::File,
    io::BufWriter,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Instant,
};
use threadpool::ThreadPool;
use wazir_drop::{
    DefaultEvaluator, Features, LongVariation, Move, Outcome, Position, Score, ScoreExpanded,
    ScoredMove, Search, Stage, WPSFeatures,
    constants::{Depth, Eval, Hyperparameters},
};
use crate::{config::FeaturesConfig, data::Sample};

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    output: PathBuf,
    num_cpus: usize,
    num_games: u64,
    batch_size: u64,
    ttable_size_mb: usize,
    pvtable_size_mb: usize,
    depth: Depth,
    extra_depth: Depth,
    temperature: f64,
    temperature_cutoff: Eval,
    features: FeaturesConfig,
}

pub fn run(config: &Config) -> Result<(), Box<dyn Error>> {
    let output = BufWriter::new(File::create(&config.output)?);
    let output = IoWrite::new(output);
    let output = Serializer::new(output).packed_format();
    let output = Arc::new(Mutex::new(output));

    match config.features {
        FeaturesConfig::PS => run_games(config, PSFeatures, &output)?,
        FeaturesConfig::WPS => run_games(config, WPSFeatures, &output)?,
    }
    Ok(())
}

fn run_games<F: Features, W: serde_cbor::ser::Write + Send + 'static>(
    config: &Config,
    features: F,
    output: &Arc<Mutex<serde_cbor::Serializer<W>>>,
) -> Result<(), Box<dyn Error>> {
    let evaluator = Arc::new(DefaultEvaluator::default());
    let thread_pool = ThreadPool::new(config.num_cpus);
    let stats = Arc::new(Mutex::new(Stats::new()));
    let start_time = Instant::now();
    log::info!(
        "Starting self-play: games={num_games}",
        num_games = config.num_games
    );
    loop {
        let cur_games = {
            let stats = stats.lock().unwrap();
            if stats.games >= config.num_games {
                break;
            }
            (config.num_games - stats.games).min(config.batch_size)
        };
        for _ in 0..cur_games {
            let config = config.clone();
            let output = output.clone();
            let evaluator = evaluator.clone();
            let stats = stats.clone();
            thread_pool.execute(
                move || match play_game(&config, &output, &evaluator, features) {
                    Ok(s) => {
                        let mut stats = stats.lock().unwrap();
                        stats.add(&s);
                    }
                    Err(e) => {
                        log::error!("Error playing game: {e}");
                        panic!("Error playing game: {e}");
                    }
                },
            );
        }
        thread_pool.join();
        {
            let stats = stats.lock().unwrap();
            log::info!(
                "games={games} / {num_games} draws={draws_percentage:.2}% moves/game = {moves_per_game:.2}\n \
                entropy/move = {entropy_per_move:.6} samples={samples} games/s={games_per_second:.2}\n  \
                pv_truncated={pv_truncated} invalid_pv={invalid_pv} ",
                games = stats.games,
                num_games = config.num_games,
                draws_percentage = stats.draws as f64 / stats.games as f64 * 100.0,
                moves_per_game = stats.moves as f64 / stats.games as f64,
                entropy_per_move = stats.entropy / stats.moves as f64,
                samples = stats.samples,
                games_per_second = stats.games as f64 / start_time.elapsed().as_secs_f64(),
                pv_truncated = stats.pv_truncated,
                invalid_pv = stats.invalid_pv,
            );
        }
    }
    Ok(())
}

fn play_game<F: Features, W: serde_cbor::ser::Write>(
    config: &Config,
    output: &Mutex<serde_cbor::Serializer<W>>,
    evaluator: &Arc<DefaultEvaluator>,
    features: F,
) -> Result<Stats, Box<dyn Error>> {
    let mut rng = StdRng::from_os_rng();
    let mut position = Position::initial();

    let hyperparameters = Hyperparameters {
        ttable_size: config.ttable_size_mb << 20,
        pvtable_size: config.pvtable_size_mb << 20,
        ..Hyperparameters::default()
    };

    let mut search = Search::new(&hyperparameters, evaluator);
    let mut stats = Stats::new();

    struct Entry {
        pv_position: Position,
        deep_score: Score,
    }
    let mut entries: Vec<Entry> = Vec::new();

    let mut prev_pv_position_hash = 0;
    let outcome = loop {
        match position.stage() {
            Stage::Setup => {
                let mov = moverand::random_setup(position.to_move(), &mut rng);
                position = position.make_setup_move(mov).unwrap();
            }
            Stage::Regular => {
                let result = search.search(
                    &position,
                    Some(config.depth),
                    None, /* deadline */
                    Some(config.temperature_cutoff),
                );
                assert!(!result.top_moves.is_empty());
                match calc_deep_score(
                    &position,
                    result.score,
                    &result.pv,
                    &mut search,
                    config.extra_depth,
                    &mut prev_pv_position_hash,
                ) {
                    Ok((pv_position, deep_score)) => {
                        entries.push(Entry {
                            pv_position,
                            deep_score,
                        });
                        stats.samples += 1;
                    }
                    Err(DeepScoreImpossible::RepeatedPVPosition) => {}
                    Err(DeepScoreImpossible::GameDecided) => {}
                    Err(DeepScoreImpossible::PVTruncated) => {
                        stats.pv_truncated += 1;
                    }
                    Err(DeepScoreImpossible::InvalidPV) => {
                        stats.invalid_pv += 1;
                    }
                }
                let (entropy, mov) = select_move(&result.top_moves, &mut rng, config.temperature);
                stats.entropy += entropy;
                stats.moves += 1;
                position = position.make_move(mov).unwrap();
            }
            Stage::End(o) => break o,
        }
    };
    stats.games += 1;
    if outcome == Outcome::Draw {
        stats.draws += 1;
    }
    let mut output = output.lock().unwrap();
    for entry in entries {
        let to_move = entry.pv_position.to_move();
        let f = [to_move, to_move.opposite()].map(|color| {
            features
                .all(&entry.pv_position, color)
                .map(|x| x as u32)
                .collect()
        });
        let deep_value = match entry.deep_score.into() {
            ScoreExpanded::Win(_) => Eval::MAX,
            ScoreExpanded::Eval(eval) => eval,
            ScoreExpanded::Loss(_) => -Eval::MAX,
        };
        let game_points = outcome.points(to_move);
        let sample = Sample {
            features: f,
            deep_value,
            game_points,
        };
        sample.serialize(&mut *output)?;
    }

    Ok(stats)
}

enum DeepScoreImpossible {
    GameDecided,
    PVTruncated,
    InvalidPV,
    RepeatedPVPosition,
}

/// Returns the PV position and the deep score.
fn calc_deep_score(
    position: &Position,
    score: Score,
    pv: &LongVariation,
    search: &mut Search<DefaultEvaluator>,
    extra_depth: Depth,
    prev_pv_position_hash: &mut u64,
) -> Result<(Position, Score), DeepScoreImpossible> {
    if !matches!(score.into(), ScoreExpanded::Eval(_)) {
        return Err(DeepScoreImpossible::GameDecided);
    }
    if pv.truncated {
        return Err(DeepScoreImpossible::PVTruncated);
    }
    let mut pv_position = position.clone();
    for &mov in pv.iter() {
        let Ok(p) = pv_position.make_move(mov) else {
            return Err(DeepScoreImpossible::InvalidPV);
        };
        pv_position = p;
    }
    let hash = pv_position.hash();
    if hash == *prev_pv_position_hash {
        return Err(DeepScoreImpossible::RepeatedPVPosition);
    }
    *prev_pv_position_hash = hash;
    let result = search.search(
        &pv_position,
        Some(extra_depth),
        None, /* deadline */
        None, /* multi_move_threshold */
    );
    Ok((pv_position, result.score))
}

// Returns (entropy, move).
fn select_move(moves: &[ScoredMove], rng: &mut StdRng, temperature: f64) -> (f64, Move) {
    let ScoreExpanded::Eval(top_eval) = moves[0].score.into() else {
        return (0.0, moves[0].mov);
    };
    let log_weight = |m: &ScoredMove| {
        let ScoreExpanded::Eval(eval) = m.score.into() else {
            return f64::NEG_INFINITY;
        };
        let rel = eval - top_eval;
        rel as f64 / temperature
    };
    let sum_weights: f64 = moves.iter().map(|m| log_weight(m).exp()).sum();
    let log_sum_weights = sum_weights.ln();
    let entropy = -1.0 / sum_weights
        * moves
            .iter()
            .map(|m| {
                let l = log_weight(m);
                l.exp() * (l - log_sum_weights)
            })
            .sum::<f64>();

    let mov = moves
        .choose_weighted(rng, |m| log_weight(m).exp())
        .unwrap()
        .mov;
    (entropy, mov)
}

struct Stats {
    games: u64,
    moves: u64,
    entropy: f64,
    draws: u64,
    samples: u64,
    pv_truncated: u64,
    invalid_pv: u64,
}

impl Stats {
    fn new() -> Self {
        Self {
            games: 0,
            moves: 0,
            entropy: 0.0,
            draws: 0,
            samples: 0,
            pv_truncated: 0,
            invalid_pv: 0,
        }
    }

    fn add(&mut self, stats: &Stats) {
        self.games += stats.games;
        self.moves += stats.moves;
        self.entropy += stats.entropy;
        self.draws += stats.draws;
        self.samples += stats.samples;
        self.pv_truncated += stats.pv_truncated;
        self.invalid_pv += stats.invalid_pv;
    }
}
