use clap::Parser;
use log::LevelFilter;
use rand::{SeedableRng, rngs::StdRng, seq::IndexedRandom};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::Deserialize;
use simplelog::{ColorChoice, CombinedLogger, TermLogger, TerminalMode, WriteLogger};
use std::{
    collections::BTreeSet,
    error::Error,
    fs::{self, File},
    path::PathBuf,
    process::ExitCode,
    sync::Arc,
};
use wazir_drop::{
    Color, DefaultEvaluator, EvaluatedPosition, Evaluator, Position, Score, ScoreExpanded,
    SetupMove, Symmetry, movegen,
};

#[derive(Parser, Debug)]
struct Args {
    config_file: PathBuf,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    log: PathBuf,
    cpus: usize,
    sample: usize,
    reasonable: usize,
    openings: usize,
    block_size: usize,
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
    let config: Config = toml::from_str(&config_text)?;
    let config_dir = args.config_file.parent().unwrap();

    let log_path = config_dir.join(&config.log);
    if let Some(log_dir) = log_path.parent() {
        fs::create_dir_all(log_dir)?;
    }
    let log_file = File::create(log_path)?;

    CombinedLogger::init(vec![
        WriteLogger::new(LevelFilter::Info, simplelog::Config::default(), log_file),
        TermLogger::new(
            LevelFilter::Info,
            simplelog::Config::default(),
            TerminalMode::Stderr,
            ColorChoice::Auto,
        ),
    ])?;

    run_with_config(&config)?;

    Ok(())
}

fn run_with_config(config: &Config) -> Result<(), Box<dyn Error>> {
    rayon::ThreadPoolBuilder::new()
        .num_threads(config.cpus)
        .build_global()
        .unwrap();

    let evaluator = Arc::new(DefaultEvaluator::default());
    let mut rng = StdRng::from_os_rng();

    log::info!("Random sample: {sample}", sample = config.sample);
    let all: Vec<SetupMove> = all_normalized_red_setup_moves().collect();
    let random_sample: Vec<SetupMove> = all
        .choose_multiple(&mut rng, config.sample)
        .copied()
        .collect();

    log::info!("Reasonable sample: {sample}", sample = config.sample,);
    let reasonable_sample: Vec<SetupMove> = find_openings(
        config.sample,
        &*evaluator,
        &all,
        &random_sample,
        config.block_size,
    )
    .into_iter()
    .map(|opening| opening.red)
    .collect();

    log::info!("Reasonable: {reasonable}", reasonable = config.reasonable);
    let reasonable: Vec<SetupMove> = find_openings(
        config.reasonable,
        &*evaluator,
        &all,
        &reasonable_sample,
        config.block_size,
    )
    .into_iter()
    .map(|opening| opening.red)
    .collect();

    log::info!(
        "Best openings: {num_openings}",
        num_openings = config.openings
    );
    let openings = find_openings(
        config.openings,
        &*evaluator,
        &reasonable,
        &reasonable,
        config.block_size,
    );
    for (index, opening) in openings.iter().enumerate() {
        log::info!(
            "{index}. {score} {red} {blue}",
            score = opening.score,
            red = opening.red,
            blue = opening.blue
        );
    }
    Ok(())
}

fn all_normalized_red_setup_moves() -> impl Iterator<Item = SetupMove> {
    movegen::setup_moves(Color::Red)
        .filter(|mov| Symmetry::normalize_red_setup(*mov).0 == Symmetry::Identity)
}

fn find_openings<E: Evaluator>(
    num_openings: usize,
    evaluator: &E,
    possible: &[SetupMove],
    reasonable: &[SetupMove],
    block_size: usize,
) -> Vec<Opening> {
    let mut openings: BTreeSet<Opening> = BTreeSet::new();
    let log_blocks = possible.len() / block_size / 30 + 1;
    for (block_index, possible_block) in possible.chunks(block_size).enumerate() {
        if block_index % log_blocks == 0 {
            log::info!("Red move index {index}", index = block_index * block_size);
        }
        let alpha = if openings.len() < num_openings {
            -Score::INFINITE
        } else {
            openings.first().unwrap().score
        };
        let results: Vec<Opening> = possible_block
            .par_iter()
            .map(|&mov| search_blue(evaluator, mov, alpha, reasonable))
            .collect();

        for opening in results {
            if opening.score > alpha {
                let inserted = openings.insert(opening);
                assert!(inserted);
            }
        }
        while openings.len() > num_openings {
            _ = openings.pop_first().unwrap();
        }
    }
    openings.into_iter().rev().collect()
}

fn search_blue<E: Evaluator>(
    evaluator: &E,
    red: SetupMove,
    alpha: Score,
    reasonable: &[SetupMove],
) -> Opening {
    let mut result = Opening {
        score: Score::INFINITE,
        red,
        blue: red,
    };
    let initial_position = EvaluatedPosition::new(evaluator, Position::initial());
    let epos_red = initial_position.make_setup_move(red).unwrap();
    'main_loop: for &reasonable_red in reasonable {
        for symmetry in [Symmetry::Identity, Symmetry::FlipX] {
            let mov = SetupMove {
                color: Color::Blue,
                pieces: symmetry.apply_to_setup(reasonable_red).pieces,
            };

            let epos_blue = epos_red.make_setup_move(mov).unwrap();
            let score = Score::from(ScoreExpanded::Eval(epos_blue.evaluate()));
            if score < result.score {
                result.score = score;
                result.blue = mov;
                if score <= alpha {
                    break 'main_loop;
                }
            }
        }
    }
    result
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct Opening {
    score: Score,
    red: SetupMove,
    blue: SetupMove,
}
