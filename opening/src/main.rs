use clap::Parser;
use log::LevelFilter;
use rand::{SeedableRng, rngs::StdRng, seq::IteratorRandom};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::Deserialize;
use simplelog::{ColorChoice, CombinedLogger, TermLogger, TerminalMode, WriteLogger};
use std::{
    collections::{BTreeSet, HashMap},
    error::Error,
    fs::{self, File},
    io::{BufWriter, Write},
    path::PathBuf,
    process::ExitCode,
    sync::Arc,
    time::Instant,
};
use wazir_drop::{
    Color, DefaultEvaluator, EvaluatedPosition, Position, Score, ScoreExpanded, SetupMove,
    Symmetry, movegen,
};

#[derive(Parser, Debug)]
struct Args {
    config_file: PathBuf,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    log: PathBuf,
    openings_file: PathBuf,
    cpus: usize,
    seed: u64,
    blue_random_sample: usize,
    reasonable_setups: Vec<usize>,
    openings: usize,
    block: usize,
    log_period_seconds: f32,
}

fn main() -> ExitCode {
    if let Err(e) = run() {
        log::error!("{e}");
        eprintln!("Error: {e}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn run() -> Result<(), Box<dyn Error>> {
    wazir_drop::log::init(wazir_drop::log::Level::Always);
    let args = Args::parse();

    let config_text = fs::read_to_string(&args.config_file)?;
    let mut config: Config = toml::from_str(&config_text)?;
    let config_dir = args.config_file.parent().unwrap();
    config.log = config_dir.join(&config.log);
    config.openings_file = config_dir.join(&config.openings_file);

    let log_file = File::create(&config.log)?;
    CombinedLogger::init(vec![
        WriteLogger::new(LevelFilter::Info, simplelog::Config::default(), log_file),
        TermLogger::new(
            LevelFilter::Info,
            simplelog::Config::default(),
            TerminalMode::Stderr,
            ColorChoice::Auto,
        ),
    ])?;

    rayon::ThreadPoolBuilder::new()
        .num_threads(config.cpus)
        .build_global()?;

    OpeningSolver::new(&config).run()?;

    Ok(())
}

struct OpeningSolver {
    config: Config,
    rng: StdRng,
    evaluator: Arc<DefaultEvaluator>,
    openings: Vec<Opening>,
    blue_setups: Vec<SetupMove>,
}

impl OpeningSolver {
    fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
            rng: StdRng::seed_from_u64(config.seed),
            evaluator: Arc::new(DefaultEvaluator::default()),
            openings: Vec::new(),
            blue_setups: Vec::new(),
        }
    }

    fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.random_sample_blue_setups();

        for num in self.config.reasonable_setups.clone() {
            self.all_openings();
            self.improve_openings(num);
            log::info!("Truncate to {num} openings");
            self.openings.truncate(num);
            self.use_openings_as_blue_setups();
        }
        self.improve_openings(self.config.openings);
        log::info!("Truncate to {num} openings", num = self.config.openings);
        self.openings.truncate(self.config.openings);
        self.print_openings()?;
        Ok(())
    }

    fn random_sample_blue_setups(&mut self) {
        log::info!(
            "Random sample {sample} blue setups",
            sample = self.config.blue_random_sample
        );
        self.blue_setups = movegen::setup_moves(Color::Blue)
            .choose_multiple(&mut self.rng, self.config.blue_random_sample);
    }

    fn use_openings_as_blue_setups(&mut self) {
        log::info!("Use openings as {} blue setups", 2 * self.openings.len());
        self.blue_setups = self
            .openings
            .iter()
            .map(|opening| opening.red.with_color(Color::Blue))
            .flat_map(|setup| {
                [Symmetry::Identity, Symmetry::FlipX].map(|symmetry| symmetry.apply_to_setup(setup))
            })
            .collect();
    }

    fn all_openings(&mut self) {
        log::info!("Generate all openings with dummy responses");
        self.openings = movegen::setup_moves(Color::Red)
            .filter(|mov| Symmetry::normalize_red_setup(*mov).0 == Symmetry::Identity)
            .map(|red| Opening {
                score: Score::DRAW,
                red,
                blue: self.blue_setups[0],
            })
            .collect();
        log::info!("Number of openings: {num}", num = self.openings.len());
    }

    fn improve_openings(&mut self, min_num_exact: usize) {
        log::info!("Calculating {min_num_exact} openings");
        let mut last_log_time = Instant::now();
        // <= min_num_exact
        let mut new_openings: BTreeSet<Opening> = BTreeSet::new();
        let mut extra_openings: BTreeSet<Opening> = BTreeSet::new();
        let mut old_openings: Vec<Opening> = Vec::new();
        for (block_index, block) in self.openings.chunks(self.config.block).enumerate() {
            if last_log_time.elapsed().as_secs_f32() >= self.config.log_period_seconds {
                log::info!(
                    "Done {done} / {all}",
                    done = block_index * self.config.block,
                    all = self.openings.len()
                );
                last_log_time = Instant::now();
            }
            let alpha = if new_openings.len() < min_num_exact {
                -Score::INFINITE
            } else {
                self.openings.first().unwrap().score
            };
            // Ok(new) if > alpha, Err(old) if <= alpha.
            let results: Vec<Result<Opening, Opening>> = block
                .par_iter()
                .map(|&opening| {
                    match compute_opening(opening.red, &self.evaluator, alpha, &self.blue_setups) {
                        Some(new) => Ok(new),
                        None => Err(opening),
                    }
                })
                .collect();
            for result in results {
                match result {
                    Ok(new) => {
                        assert!(new_openings.insert(new));
                        if new_openings.len() > min_num_exact {
                            let worst = new_openings.pop_first().unwrap();
                            assert!(extra_openings.insert(worst));
                        }
                    }
                    Err(old) => {
                        old_openings.push(old);
                    }
                }
            }
        }
        self.openings = new_openings.iter().rev().copied().collect();
        self.openings.extend(extra_openings.iter().rev().copied());
        self.openings.extend(old_openings.iter().rev().copied());
        log::info!(
            "Exact openings: {exact} / {all}",
            exact = new_openings.len() + extra_openings.len(),
            all = self.openings.len()
        );
    }

    fn print_openings(&self) -> Result<(), Box<dyn Error>> {
        let file = File::create(&self.config.openings_file)?;
        let mut writer = BufWriter::new(file);

        let setup_number_mapping: HashMap<SetupMove, usize> = self
            .openings
            .iter()
            .enumerate()
            .map(|(index, opening)| (opening.red, index))
            .collect();

        for (index, opening) in self.openings.iter().enumerate() {
            let (symmetry, red_equivalent) =
                Symmetry::normalize_red_setup(opening.blue.with_color(Color::Red));
            let setup_number = setup_number_mapping
                .get(&red_equivalent)
                .copied()
                .map(|index| index.to_string())
                .unwrap_or("none".to_string());
            writeln!(
                writer,
                "{index}. {score} {red} {blue} ({setup_number}, {symmetry})",
                score = opening.score,
                red = opening.red,
                blue = opening.blue,
            )?;
        }
        Ok(())
    }
}

#[cfg(false)]
fn run_with_config(config: &Config) -> Result<(), Box<dyn Error>> {}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct Opening {
    score: Score,
    red: SetupMove,
    blue: SetupMove,
}

// Only return something if score > alpha.
fn compute_opening(
    red: SetupMove,
    evaluator: &DefaultEvaluator,
    alpha: Score,
    blue_setups: &[SetupMove],
) -> Option<Opening> {
    let mut result = Opening {
        score: Score::INFINITE,
        red,
        blue: blue_setups[0],
    };
    let epos0 = EvaluatedPosition::new(evaluator, Position::initial());
    let epos1 = epos0.make_setup_move(red).unwrap();
    for &blue_setup in blue_setups {
        let epos2 = epos1.make_setup_move(blue_setup).unwrap();
        let score = Score::from(ScoreExpanded::Eval(epos2.evaluate()));
        if score < result.score {
            if score <= alpha {
                return None;
            }
            result.score = score;
            result.blue = blue_setup;
        }
    }
    Some(result)
}
