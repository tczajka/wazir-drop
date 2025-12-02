use extra::Nnue;
use std::{process::ExitCode, sync::Arc};
use wazir_drop::{MainPlayerFactory, constants::Hyperparameters, run_cli};

fn main() -> ExitCode {
    let player_factory =
        MainPlayerFactory::new(&Hyperparameters::default(), &Arc::new(Nnue::default()));
    run_cli(&player_factory)
}
