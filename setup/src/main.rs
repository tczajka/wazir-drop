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
    config_file: PathBuf,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    log: PathBuf,
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

    compute_setups(&config)?;

    Ok(())
}

fn compute_setups(config: &Config) -> Result<(), Box<dyn Error>> {
    Ok(())
}
