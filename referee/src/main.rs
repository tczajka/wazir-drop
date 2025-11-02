use clap::Parser;
use external_player::ExternalPlayerFactory;
use log::LevelFilter;
use rand::{SeedableRng, rngs::StdRng};
use random_player::RandomPlayerFactory;
use referee::run_match;
use serde::Deserialize;
use simplelog::{ColorChoice, CombinedLogger, TermLogger, TerminalMode, WriteLogger};
use std::{
    collections::HashMap,
    error::Error,
    fs::{self, File},
    path::{Path, PathBuf},
    process::ExitCode,
    sync::Arc,
    time::Duration,
};
use wazir_drop::{MainPlayerFactory, PlayerFactory};

#[derive(Parser, Debug)]
struct Args {
    config: PathBuf,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    log_dir: PathBuf,
    num_cpus: usize,
    player: HashMap<String, PlayerConfig>,
    r#match: Vec<MatchConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum PlayerConfig {
    Main,
    Random,
    External { path: PathBuf },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MatchConfig {
    players: [String; 2],
    opening_length: usize,
    num_rounds: usize,
    time_limit_0: Option<u32>,
    time_limit_1: Option<u32>,
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
    let log_dir = config_dir.join(&config.log_dir);
    fs::create_dir_all(&log_dir)?;
    let log_file = File::create(log_dir.join("referee.log"))?;

    CombinedLogger::init(vec![
        WriteLogger::new(LevelFilter::Info, simplelog::Config::default(), log_file),
        TermLogger::new(
            LevelFilter::Info,
            simplelog::Config::default(),
            TerminalMode::Stderr,
            ColorChoice::Auto,
        ),
    ])?;

    run_matches(&config, config_dir, &log_dir)?;
    Ok(())
}

fn run_matches(config: &Config, config_dir: &Path, log_dir: &Path) -> Result<(), Box<dyn Error>> {
    let player_factories: HashMap<String, Arc<dyn PlayerFactory>> = config
        .player
        .iter()
        .map(|(name, player_config)| {
            let player_factory: Arc<dyn PlayerFactory> = match player_config {
                PlayerConfig::Main => Arc::new(MainPlayerFactory::default()),
                PlayerConfig::Random => Arc::new(RandomPlayerFactory::new()),
                PlayerConfig::External { path } => Arc::new(ExternalPlayerFactory::new(
                    name,
                    &config_dir.join(path),
                    log_dir,
                )),
            };
            (name.clone(), player_factory)
        })
        .collect();

    for match_config in config.r#match.iter() {
        for player_name in match_config.players.iter() {
            if !player_factories.contains_key(player_name) {
                return Err(format!("Player {player_name} not found").into());
            }
        }
    }

    let mut rng = StdRng::from_os_rng();

    for (match_idx, match_config) in config.r#match.iter().enumerate() {
        let match_id = format!("{match_idx}");
        log::info!("Match {match_id}");

        let player_factories = match_config
            .players
            .each_ref()
            .map(|name| player_factories.get(name).unwrap().clone());

        let time_limits = [match_config.time_limit_0, match_config.time_limit_1]
            .map(|t| t.map(|t| Duration::from_millis(t.into())));

        let match_result = run_match(
            &match_id,
            match_config.num_rounds,
            config.num_cpus,
            match_config.opening_length,
            player_factories,
            time_limits,
            &mut rng,
        );
        log::info!("{match_result}");
    }
    Ok(())
}
