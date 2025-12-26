use clap::Parser;
use log::LevelFilter;
use rand::{Rng, SeedableRng, rngs::StdRng};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::Deserialize;
use simplelog::{ColorChoice, CombinedLogger, TermLogger, TerminalMode, WriteLogger};
use std::{
    array,
    error::Error,
    fs::{self, File},
    path::PathBuf,
    process::ExitCode,
    sync::Arc,
    time::{Duration, Instant},
};
use wazir_drop::{
    AnyMove, Color, DefaultEvaluator, MainPlayerFactory, PlayerFactory, constants::Hyperparameters,
    enums::EnumMap,
};

#[derive(Parser, Debug)]
struct Args {
    config: PathBuf,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    log: PathBuf,
    cpus: usize,
    rounds: u64,
    batch: u64,
    skip_rounds: u64,
    initial_delta: f64,
    delta_exponent: f64,
    initial_learning_rate: f64,
    learning_rate_exponent: f64,
    time_limit_ms: u64,
    parameter: [ParameterConfig; NUM_PARAMETERS],
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ParameterConfig {
    name: String,
    min: Option<f64>,
    max: Option<f64>,
    // transform(scale * x)
    transform: Transform,
    scale: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Transform {
    Identity,
    Exp,
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
    let config_text = fs::read_to_string(&args.config)?;
    let config: Config = toml::from_str(&config_text)?;
    let config_dir = args.config.parent().unwrap();
    let log_path = config_dir.join(&config.log);
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

    rayon::ThreadPoolBuilder::new()
        .num_threads(config.cpus)
        .build_global()?;

    run_tune(&config);

    Ok(())
}

fn run_tune(config: &Config) {
    log::info!("Tuning hyperparameters");
    let mut rng = StdRng::from_os_rng();
    let mut parameters = from_hyperparameters(config, &Hyperparameters::default());
    let mut num_rounds = 0;
    let evaluator = Arc::new(DefaultEvaluator::default());
    let start_time = Instant::now();
    log_parameters(&config, &parameters);
    while num_rounds < config.rounds {
        let next_num_rounds = (num_rounds + config.batch).min(config.rounds);
        // 1 = skip rounds
        // 2 = 2 * skip_rounds
        let time = 1.0 + (num_rounds as f64 / config.skip_rounds as f64);
        let delta_size = config.initial_delta / time.powf(config.delta_exponent);
        let learning_rate = config.initial_learning_rate / time.powf(config.learning_rate_exponent);
        run_batch(
            &mut parameters,
            config,
            &evaluator,
            next_num_rounds - num_rounds,
            delta_size,
            learning_rate,
            &mut rng,
        );
        num_rounds = next_num_rounds;

        log::info!(
            "Rounds: {num_rounds}/{config_rounds} rounds/s={rounds_per_second:.3}",
            config_rounds = config.rounds,
            rounds_per_second = num_rounds as f64 / start_time.elapsed().as_secs_f64(),
        );
        log_parameters(&config, &parameters);
    }
    log::info!("Results");
    for (i, &param) in parameters.iter().enumerate() {
        let c = &config.parameter[i];
        let value = unnormalize(c, param);
        log::info!("{name}: {value:.6}", name = c.name, value = value);
    }
}

fn log_parameters(config: &Config, parameters: &Parameters) {
    let mut param_str = String::new();
    for (i, &param) in parameters.iter().enumerate() {
        let x = unnormalize(&config.parameter[i], param);
        param_str.push_str(&format!("{x:.6}, "));
    }
    log::info!("Parameters: {param_str}");
}

const NUM_PARAMETERS: usize = 9;
type Parameters = [f64; NUM_PARAMETERS];

fn from_hyperparameters(config: &Config, hyperparameters: &Hyperparameters) -> Parameters {
    let unnormalized = [
        hyperparameters.null_move_margin,
        hyperparameters.futility_margin,
        hyperparameters.time_reduction_per_move,
        hyperparameters.time_reduction_per_late_move,
        hyperparameters.soft_time_fraction,
        hyperparameters.start_next_depth_fraction,
        hyperparameters.panic_eval_threshold,
        hyperparameters.panic_multiplier,
        hyperparameters.panic_max_remaining,
    ];
    array::from_fn(|i| normalize(&config.parameter[i], unnormalized[i]))
}

fn to_hyperparameters(config: &Config, parameters: &Parameters) -> Hyperparameters {
    let unnormalized: [f64; NUM_PARAMETERS] =
        array::from_fn(|i| unnormalize(&config.parameter[i], parameters[i]));
    Hyperparameters {
        contempt: 0.0,
        null_move_margin: unnormalized[0],
        futility_margin: unnormalized[1],
        time_reduction_per_move: unnormalized[2],
        time_reduction_per_late_move: unnormalized[3],
        soft_time_fraction: unnormalized[4],
        start_next_depth_fraction: unnormalized[5],
        panic_eval_threshold: unnormalized[6],
        panic_multiplier: unnormalized[7],
        panic_max_remaining: unnormalized[8],
        ..Hyperparameters::default()
    }
}

fn add_parameters(a: &Parameters, b: &Parameters) -> Parameters {
    array::from_fn(|i| a[i] + b[i])
}

fn mul_parameters(a: f64, b: &Parameters) -> Parameters {
    array::from_fn(|i| a * b[i])
}

fn sub_parameters(a: &Parameters, b: &Parameters) -> Parameters {
    array::from_fn(|i| a[i] - b[i])
}

fn normalize(parameter: &ParameterConfig, value: f64) -> f64 {
    let x = match parameter.transform {
        Transform::Identity => value,
        Transform::Exp => value.ln(),
    };
    x / parameter.scale
}

fn unnormalize(parameter: &ParameterConfig, value: f64) -> f64 {
    let x = value * parameter.scale;
    match parameter.transform {
        Transform::Identity => x,
        Transform::Exp => x.exp(),
    }
}

fn run_batch(
    parameters: &mut Parameters,
    config: &Config,
    evaluator: &Arc<DefaultEvaluator>,
    num_rounds: u64,
    delta_size: f64,
    learning_rate: f64,
    rng: &mut StdRng,
) {
    let round_configs: Vec<RoundConfig> = (0..num_rounds)
        .map(|_| RoundConfig::new(rng, delta_size))
        .collect();
    let gradients: Vec<Parameters> = round_configs
        .par_iter()
        .map(|round_config| run_round(parameters, round_config, config, evaluator))
        .collect();
    for gradient in &gradients {
        *parameters = add_parameters(parameters, &mul_parameters(learning_rate, gradient));
    }
    for (i, p) in parameters.iter_mut().enumerate() {
        let c = &config.parameter[i];
        if let Some(min) = c.min {
            *p = p.max(normalize(c, min));
        }
        if let Some(max) = c.max {
            *p = p.min(normalize(c, max));
        }
    }
}

fn random_delta(delta_size: f64, rng: &mut StdRng) -> Parameters {
    array::from_fn(|_| {
        let sign: i32 = rng.random_range(0..2) * 2 - 1;
        sign as f64 * delta_size
    })
}

struct RoundConfig {
    delta: Parameters,
    opening: Vec<AnyMove>,
}

impl RoundConfig {
    fn new(rng: &mut StdRng, delta_size: f64) -> Self {
        let delta = random_delta(delta_size, rng);
        let opening = referee::random_opening(2, rng);
        Self { delta, opening }
    }
}

// Returns the estimated gradient of parameters.
fn run_round(
    parameters: &Parameters,
    round_config: &RoundConfig,
    config: &Config,
    evaluator: &Arc<DefaultEvaluator>,
) -> Parameters {
    let hyper_plus = to_hyperparameters(config, &add_parameters(parameters, &round_config.delta));
    let player_plus = MainPlayerFactory::new(&hyper_plus, evaluator);
    let hyper_minus = to_hyperparameters(config, &sub_parameters(parameters, &round_config.delta));
    let player_minus = MainPlayerFactory::new(&hyper_minus, evaluator);
    let time_limits = EnumMap::from_fn(|_| Some(Duration::from_millis(config.time_limit_ms)));

    let player_factories = EnumMap::from_fn(|color| match color {
        Color::Red => &player_plus as &dyn PlayerFactory,
        Color::Blue => &player_minus as &dyn PlayerFactory,
    });
    let points0 = referee::run_game("", player_factories, &round_config.opening, time_limits)
        .outcome
        .points(Color::Red);

    let player_factories = EnumMap::from_fn(|color| match color {
        Color::Red => &player_minus as &dyn PlayerFactory,
        Color::Blue => &player_plus as &dyn PlayerFactory,
    });
    let points1 = referee::run_game("", player_factories, &round_config.opening, time_limits)
        .outcome
        .points(Color::Blue);

    let points = (points0 + points1) as f64;
    array::from_fn(|i| points / (2.0 * round_config.delta[i]))
}
