use clap::Parser;
use log::LevelFilter;
use rand::{SeedableRng, rngs::StdRng, seq::IndexedRandom};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::Deserialize;
use simplelog::{ColorChoice, CombinedLogger, TermLogger, TerminalMode, WriteLogger};
use std::{
    collections::{BTreeSet, HashMap, HashSet},
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
    Symmetry, base128::Base128Encoder, book::encode_setup_move, constants::Hyperparameters,
    movegen,
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
    export_book: PathBuf,
    cpus: usize,
    ttable_size_mb: usize,
    pvtable_size_mb: usize,
    seed: u64,
    sample_size: usize,
    sample_iterations: usize,
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
    config.export_book = config_dir.join(&config.export_book);

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
    hyperparameters: Hyperparameters,
    all_openings: Vec<Opening>,
    openings: Vec<Opening>,
    blue_setups: Vec<SetupMove>,
}

impl OpeningSolver {
    fn new(config: &Config) -> Self {
        let hyperparameters = Hyperparameters {
            ttable_size: config.ttable_size_mb << 20,
            pvtable_size: config.pvtable_size_mb << 20,
            ..Hyperparameters::default()
        };
        Self {
            config: config.clone(),
            rng: StdRng::seed_from_u64(config.seed),
            evaluator: Arc::new(DefaultEvaluator::default()),
            hyperparameters,
            all_openings: Vec::new(),
            openings: Vec::new(),
            blue_setups: Vec::new(),
        }
    }

    fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.generate_all_openings();
        self.random_sample_openings();
        self.use_openings_as_blue_setups();

        for iteration in 0..self.config.sample_iterations {
            log::info!(
                "Sample iteration {iteration} / {n}",
                n = self.config.sample_iterations
            );
            self.improve_openings(self.config.sample_size);
            self.use_openings_as_blue_setups();
        }
        self.print_openings()?;
        self.export_openings()?;
        Ok(())
    }

    fn random_sample_openings(&mut self) {
        let n = self.config.sample_size;
        log::info!("Random sample {n} openings");
        self.openings = self
            .all_openings
            .choose_multiple(&mut self.rng, n)
            .copied()
            .collect();
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

    fn generate_all_openings(&mut self) {
        log::info!("Generate all openings");
        self.all_openings = movegen::setup_moves(Color::Red)
            .filter(|mov| Symmetry::normalize_red_setup(*mov).0 == Symmetry::Identity)
            .map(|red| Opening {
                score: Score::DRAW,
                red,
                blue: None,
            })
            .collect();
        log::info!("Number of openings: {num}", num = self.all_openings.len());
    }

    fn improve_openings(&mut self, n: usize) {
        log::info!("Calculating {n} openings");
        let mut last_log_time = Instant::now();
        let mut new_openings: BTreeSet<Opening> = BTreeSet::new();
        for (block_index, block) in self.all_openings.chunks(self.config.block).enumerate() {
            if last_log_time.elapsed().as_secs_f32() >= self.config.log_period_seconds {
                log::info!(
                    "Done {done} / {all}",
                    done = block_index * self.config.block,
                    all = self.all_openings.len()
                );
                last_log_time = Instant::now();
            }
            let alpha = if new_openings.len() < n {
                -Score::INFINITE
            } else {
                new_openings.first().unwrap().score
            };
            let new: Vec<Opening> = block
                .par_iter()
                .filter_map(|opening| {
                    compute_opening_eval(opening.red, &self.evaluator, alpha, &self.blue_setups)
                })
                .collect();
            for opening in new {
                assert!(new_openings.insert(opening));
                if new_openings.len() > n {
                    _ = new_openings.pop_first().unwrap();
                }
            }
        }
        let new_openings: Vec<Opening> = new_openings.into_iter().rev().collect();
        let overlap = calculate_overlap(&self.openings, &new_openings);
        log::info!("Overlap with previous: {overlap} / {n}");
        self.openings = new_openings;
    }

    fn print_openings(&self) -> Result<(), Box<dyn Error>> {
        log::info!("Print openings to {}", self.config.openings_file.display());
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
                Symmetry::normalize_red_setup(opening.blue.unwrap().with_color(Color::Red));
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
                blue = opening.blue.unwrap(),
            )?;
        }
        Ok(())
    }

    fn export_openings(&self) -> Result<(), Box<dyn Error>> {
        log::info!("Export openings to {}", self.config.export_book.display());

        let mut encoder = Base128Encoder::new();
        for opening in &self.openings {
            encode_setup_move(&mut encoder, opening.red);
            encode_setup_move(&mut encoder, opening.blue.unwrap());
        }
        let encoded = encoder.finish();

        let file = File::create(&self.config.export_book)?;
        let mut writer = BufWriter::new(file);
        writeln!(
            writer,
            "pub const NUM_OPENINGS: usize = {};",
            self.openings.len()
        )?;
        writeln!(writer, "pub const OPENINGS: &str = r\"{}\";", encoded)?;
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct Opening {
    score: Score,
    red: SetupMove,
    blue: Option<SetupMove>,
}

// Only return something if score > alpha.
fn compute_opening_eval(
    red: SetupMove,
    evaluator: &DefaultEvaluator,
    alpha: Score,
    blue_setups: &[SetupMove],
) -> Option<Opening> {
    let mut result = Opening {
        score: Score::INFINITE,
        red,
        blue: None,
    };
    let pos0 = EvaluatedPosition::new(evaluator, Position::initial());
    let pos1 = pos0.make_setup_move(red).unwrap();
    for &blue_setup in blue_setups {
        let pos2 = pos1.make_setup_move(blue_setup).unwrap();
        let score = Score::from(ScoreExpanded::Eval(pos2.evaluate()));
        if score < result.score {
            if score <= alpha {
                return None;
            }
            result.score = score;
            result.blue = Some(blue_setup);
        }
    }
    Some(result)
}

fn calculate_overlap(a: &[Opening], b: &[Opening]) -> usize {
    let s: HashSet<SetupMove> = a.iter().map(|opening| opening.red).collect();
    b.iter().filter(|opening| s.contains(&opening.red)).count()
}
