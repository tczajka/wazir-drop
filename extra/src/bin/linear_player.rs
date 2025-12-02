use extra::LinearEvaluator;
use std::{process::ExitCode, sync::Arc};
use wazir_drop::{MainPlayerFactory, WPSFeatures, constants::Hyperparameters, run_cli};

fn main() -> ExitCode {
    let player_factory = MainPlayerFactory::new(
        &Hyperparameters::default(),
        &Arc::new(LinearEvaluator::<WPSFeatures>::default()),
    );
    run_cli(&player_factory)
}
