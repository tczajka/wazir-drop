use extra::moverand;
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
    DefaultEvaluator, Features, PSFeatures, Position, Score, ScoreExpanded, Search, Stage,
    TopVariation, constants::Hyperparameters,
};

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    output: PathBuf,
    num_cpus: usize,
    num_games: u64,
    batch_size: u64,
    ttable_size_mb: usize,
    depth: u16,
    extra_depth: u16,
    temperature: i32,
    temperature_cutoff: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Sample {
    /// [to move, other]
    features: [Vec<u32>; 2],
    /// Value from deeper search.
    // i32::MAX is win, -i32::MAX is loss
    deep_value: i32,
    /// +1 = win, -1 = loss
    game_points: i32,
}

pub fn run(config: &Config) -> Result<(), Box<dyn Error>> {
    let output = BufWriter::new(File::create(&config.output)?);
    let output = IoWrite::new(output);
    let output = Serializer::new(output).packed_format();
    let output = Arc::new(Mutex::new(output));

    run_games(config, PSFeatures, &output)?;
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
    loop {
        let cur_games = {
            let stats = stats.lock().unwrap();
            log::info!(
                "games {games} samples {samples} games/s {games_per_second:.2} \
                pv_truncated {pv_truncated} invalid_pv {invalid_pv}",
                games = stats.games,
                samples = stats.samples,
                games_per_second = stats.games as f64 / start_time.elapsed().as_secs_f64(),
                pv_truncated = stats.pv_truncated,
                invalid_pv = stats.invalid_pv
            );
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
            let features = features.clone();
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
                let variations = search.search_top_variations(
                    &position,
                    config.depth,
                    config.temperature_cutoff,
                );
                log::debug!("num variations: {}", variations.len());
                assert!(!variations.is_empty());
                match calc_deep_score(
                    &position,
                    &variations[0],
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
                let mov = select_variation(&variations, &mut rng, config.temperature)
                    .variation
                    .moves[0];
                position = position.make_regular_move(mov).unwrap();
            }
            Stage::End(o) => break o,
        }
    };
    stats.games += 1;
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
            ScoreExpanded::Win(_) => i32::MAX,
            ScoreExpanded::Eval(eval) => eval,
            ScoreExpanded::Loss(_) => -i32::MAX,
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
    pv: &TopVariation,
    search: &mut Search<DefaultEvaluator>,
    extra_depth: u16,
    prev_pv_position_hash: &mut u64,
) -> Result<(Position, Score), DeepScoreImpossible> {
    if !matches!(pv.score.into(), ScoreExpanded::Eval(_)) {
        return Err(DeepScoreImpossible::GameDecided);
    }
    if pv.variation.truncated {
        return Err(DeepScoreImpossible::PVTruncated);
    }
    let mut pv_position = position.clone();
    for &mov in pv.variation.moves.iter() {
        let Ok(p) = pv_position.make_regular_move(mov) else {
            return Err(DeepScoreImpossible::InvalidPV);
        };
        pv_position = p;
    }
    let hash = pv_position.hash();
    if hash == *prev_pv_position_hash {
        return Err(DeepScoreImpossible::RepeatedPVPosition);
    }
    *prev_pv_position_hash = hash;
    let result = search.search(&pv_position, Some(extra_depth), None);
    let mut deep_score = result.score;
    if pv_position.to_move() != position.to_move() {
        deep_score = -deep_score;
    }
    Ok((pv_position, deep_score))
}

fn select_variation<'a>(
    variations: &'a [TopVariation],
    rng: &mut StdRng,
    temperature: i32,
) -> &'a TopVariation {
    let ScoreExpanded::Eval(top_eval) = variations[0].score.into() else {
        return &variations[0];
    };
    variations
        .choose_weighted(rng, |v| {
            let ScoreExpanded::Eval(eval) = v.score.into() else {
                return 0.0;
            };
            let rel = eval - top_eval;
            let log_prob = rel as f64 / temperature as f64;
            log_prob.exp()
        })
        .unwrap()
}

struct Stats {
    games: u64,
    samples: u64,
    pv_truncated: u64,
    invalid_pv: u64,
}

impl Stats {
    fn new() -> Self {
        Self {
            games: 0,
            samples: 0,
            pv_truncated: 0,
            invalid_pv: 0,
        }
    }

    fn add(&mut self, stats: &Stats) {
        self.games += stats.games;
        self.samples += stats.samples;
        self.pv_truncated += stats.pv_truncated;
        self.invalid_pv += stats.invalid_pv;
    }
}
