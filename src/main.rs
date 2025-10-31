use wazir_drop::{run_cli, MainPlayerFactory};

use std::process::ExitCode;

fn main() -> ExitCode {
    run_cli(&MainPlayerFactory::default())
}
