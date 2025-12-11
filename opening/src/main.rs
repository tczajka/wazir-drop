use clap::Parser;
use log::LevelFilter;
use rand::{SeedableRng, rngs::StdRng, seq::IteratorRandom};
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
    sample: usize,
    reasonable: usize,
    openings: usize,
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
    let mut rng = StdRng::from_os_rng();
    log::info!("Sampling {sample} setup moves", sample = config.sample);
    let sample = all_normalized_red_setup_moves().choose_multiple(&mut rng, config.sample);
    log::info!(
        "Finding {reasonable} reasonable openings",
        reasonable = config.reasonable
    );

    let reasonable: Vec<SetupMove> = find_openings(
        config.reasonable,
        &*evaluator,
        all_normalized_red_setup_moves(),
        &sample,
    )
    .into_iter()
    .map(|opening| opening.red)
    .collect();

    log::info!(
        "Finding {num_openings} openings",
        num_openings = config.openings
    );
    let openings = find_openings(
        config.openings,
        &*evaluator,
        reasonable.iter().copied(),
        &reasonable,
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
    possible: impl Iterator<Item = SetupMove>,
    reasonable: &[SetupMove],
) -> Vec<Opening> {
    let evaluated_position = EvaluatedPosition::new(evaluator, Position::initial());
    let mut openings: BTreeSet<Opening> = BTreeSet::new();
    for mov in possible {
        let epos2 = evaluated_position.make_setup_move(mov).unwrap();
        let alpha = if openings.len() < num_openings {
            -Score::INFINITE
        } else {
            openings.first().unwrap().score
        };
        let result = search_blue(&epos2, -alpha, reasonable);
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
    }
    openings.into_iter().rev().collect()
}

fn search_blue<E: Evaluator>(
    evaluated_position: &EvaluatedPosition<E>,
    beta: Score,
    reasonable: &[SetupMove],
) -> BlueResult {
    let mut result = BlueResult {
        score: -Score::INFINITE,
        mov: movegen::setup_moves(Color::Blue).next().unwrap(),
    };
    for &red_mov in reasonable {
        for symmetry in [Symmetry::Identity, Symmetry::FlipX] {
            let mov = SetupMove {
                color: Color::Blue,
                pieces: symmetry.apply_to_setup(red_mov).pieces,
            };

            let epos2 = evaluated_position.make_setup_move(mov).unwrap();
            let score = -Score::from(ScoreExpanded::Eval(epos2.evaluate()));
            if score > result.score {
                result = BlueResult { score, mov };
                if score >= beta {
                    break;
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

#[derive(Debug, Copy, Clone)]
struct BlueResult {
    score: Score,
    mov: SetupMove,
}
