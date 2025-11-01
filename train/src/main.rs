mod self_play;

use clap::{Parser, Subcommand};
use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, TermLogger, TerminalMode, WriteLogger};
use std::{
    error::Error,
    fs::{self, File},
    path::Path,
    process::ExitCode,
};

#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    SelfPlay(self_play::Args),
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
    match args.command {
        Command::SelfPlay(args) => self_play::run(args)?,
    }
    Ok(())
}

fn init_log(log_dir: impl AsRef<Path>, log_name: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(log_dir.as_ref())?;
    let log_file = File::create(log_dir.as_ref().join(log_name.as_ref()))?;
    CombinedLogger::init(vec![
        WriteLogger::new(LevelFilter::Info, simplelog::Config::default(), log_file),
        TermLogger::new(
            LevelFilter::Info,
            simplelog::Config::default(),
            TerminalMode::Stderr,
            ColorChoice::Auto,
        ),
    ])?;
    Ok(())
}
