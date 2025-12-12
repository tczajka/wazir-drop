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
    Color, DefaultEvaluator, Position, Score, Search, SetupMove, Symmetry,
    base128::Base128Encoder,
    book::encode_setup_move,
    constants::{Depth, Hyperparameters, ONE_PLY},
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
    ttable_size_kb: usize,
    pvtable_size_kb: usize,
    seed: u64,
    blue_random_sample: usize,
    reasonable_setups: Vec<usize>,
    /// How many to compute with depth 1, 2, etc.
    openings: Vec<usize>,
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
    hyperparameters: Hyperparameters,
    openings: Vec<Opening>,
    blue_setups: Vec<SetupMove>,
}

impl OpeningSolver {
    fn new(config: &Config) -> Self {
        let hyperparameters = Hyperparameters {
            ttable_size: config.ttable_size_kb << 10,
            pvtable_size: config.pvtable_size_kb << 10,
            ..Hyperparameters::default()
        };
        Self {
            config: config.clone(),
            rng: StdRng::seed_from_u64(config.seed),
            hyperparameters,
            openings: Vec::new(),
            blue_setups: Vec::new(),
        }
    }

    fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.random_sample_blue_setups();

        let mut depth = ONE_PLY;
        for num in self.config.reasonable_setups.clone() {
            self.all_openings();
            self.improve_openings(num, depth);
            log::info!("Truncate to {num} openings");
            self.openings.truncate(num);
            self.use_openings_as_blue_setups();
        }
        for num in self.config.openings.clone() {
            log::info!("Calculate {num} openings at depth {depth}");
            self.improve_openings(num, depth);
            self.use_openings_as_blue_setups();
            depth += ONE_PLY;
        }
        log::info!("Truncate to {num} openings", num = self.config.openings[0]);
        self.openings.truncate(self.config.openings[0]);
        self.print_openings()?;
        self.export_openings()?;
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

    fn improve_openings(&mut self, min_num_exact: usize, depth: Depth) {
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
                new_openings.first().unwrap().score
            };
            // Ok(new) if > alpha, Err(old) if <= alpha.
            let results: Vec<Result<Opening, Opening>> = block
                .par_iter()
                .map(|&opening| {
                    match compute_opening(
                        opening.red,
                        &self.hyperparameters,
                        depth,
                        alpha,
                        &self.blue_setups,
                    ) {
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

    fn export_openings(&self) -> Result<(), Box<dyn Error>> {
        log::info!("Export openings to {}", self.config.export_book.display());

        let mut encoder = Base128Encoder::new();
        for opening in &self.openings {
            encode_setup_move(&mut encoder, opening.red);
            encode_setup_move(&mut encoder, opening.blue);
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
    blue: SetupMove,
}

// Only return something if score > alpha.
fn compute_opening(
    red: SetupMove,
    hyperparameters: &Hyperparameters,
    depth: Depth,
    alpha: Score,
    blue_setups: &[SetupMove],
) -> Option<Opening> {
    let mut result = Opening {
        score: Score::INFINITE,
        red,
        blue: blue_setups[0],
    };
    let pos0 = Position::initial();
    let pos1 = pos0.make_setup_move(red).unwrap();
    let mut search = Search::new(hyperparameters, &Arc::new(DefaultEvaluator::default()));
    for &blue_setup in blue_setups {
        let pos2 = pos1.make_setup_move(blue_setup).unwrap();
        let result2 = search.search(&pos2, Some(depth), None, None);
        if result2.score < result.score {
            if result2.score <= alpha {
                return None;
            }
            result.score = result2.score;
            result.blue = blue_setup;
        }
    }
    Some(result)
}
