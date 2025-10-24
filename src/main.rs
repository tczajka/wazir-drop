use std::process::ExitCode;
use wazir_drop::{run_cli, MainPlayerFactory};

fn main() -> ExitCode {
    run_cli(&MainPlayerFactory::new())
}
