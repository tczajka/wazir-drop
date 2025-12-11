use clap::Parser;
use log::LevelFilter;
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
    num_openings: usize,
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
    let evaluator = Arc::new(DefaultEvaluator::default());
    let openings = compute_openings_eval(&*evaluator, config.num_openings);
    for (index, opening) in openings.iter().enumerate() {
        log::info!(
            "{index}. {red} {blue}",
            red = opening.red,
            blue = opening.blue
        );
    }
    Ok(())
}

fn compute_openings_eval<E: Evaluator>(evaluator: &E, num_openings: usize) -> Vec<Opening> {
    let evaluated_position = EvaluatedPosition::new(evaluator, Position::initial());
    let mut openings: BTreeSet<Opening> = BTreeSet::new();
    let mut move_index = 0;
    for mov in movegen::setup_moves(Color::Red) {
        let (symmetry, mov) = Symmetry::normalize_red_setup(mov);
        if symmetry != Symmetry::Identity {
            continue;
        }
        log::info!("red move {move_index}");
        let evaluated_position = evaluated_position.make_setup_move(mov).unwrap();
        let alpha = if openings.len() < num_openings {
            -Score::INFINITE
        } else {
            openings.first().unwrap().score
        };
        let result = search_blue_setup_eval(&evaluated_position, -alpha);
        let score = -result.score;
        if score > alpha {
            let inserted = openings.insert(Opening {
                score,
                red: mov,
                blue: result.mov,
            });
            assert!(inserted);
            if openings.len() > num_openings {
                _ = openings.pop_first().unwrap();
            }
        }
        move_index += 1;
    }
    openings.into_iter().rev().collect()
}

fn search_blue_setup_eval<E: Evaluator>(
    evaluated_position: &EvaluatedPosition<E>,
    beta: Score,
) -> BlueSetup {
    let mut result = BlueSetup {
        score: -Score::INFINITE,
        mov: movegen::setup_moves(Color::Blue).next().unwrap(),
    };
    for mov in movegen::setup_moves(Color::Blue) {
        let epos2 = evaluated_position.make_setup_move(mov).unwrap();
        let score = -Score::from(ScoreExpanded::Eval(epos2.evaluate()));
        if score > result.score {
            result = BlueSetup { score, mov };
            if score >= beta {
                break;
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

#[derive(Debug, Copy, Clone)]
struct BlueSetup {
    score: Score,
    mov: SetupMove,
}
