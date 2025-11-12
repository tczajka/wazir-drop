mod export;
mod learn;
mod linear;
mod model;
mod nnue;
mod self_play;

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
    command: Vec<Command>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
enum Command {
    SelfPlay(self_play::Config),
    Learn(learn::Config),
    Export(export::Config),
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

    let log_file = File::create(config_dir.join(&config.log))?;
    CombinedLogger::init(vec![
        WriteLogger::new(LevelFilter::Info, simplelog::Config::default(), log_file),
        TermLogger::new(
            LevelFilter::Info,
            simplelog::Config::default(),
            TerminalMode::Stderr,
            ColorChoice::Auto,
        ),
    ])?;

    for command in &config.command {
        match command {
            Command::SelfPlay(config) => self_play::run(config)?,
            Command::Learn(config) => learn::run(config)?,
            Command::Export(config) => export::run(config)?,
        }
    }

    Ok(())
}
