use clap::Parser;
use log::LevelFilter;
use serde::Deserialize;
use simplelog::{ColorChoice, CombinedLogger, TermLogger, TerminalMode, WriteLogger};
use std::{
    error::Error,
    fs::{self, File},
    path::PathBuf,
    process::ExitCode,
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

fn run_tune(_config: &Config) {}
