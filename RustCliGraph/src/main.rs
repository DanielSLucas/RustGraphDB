use rustCli::lib::api::cli::run_cli;
use std::env;
use std::sync::{Arc, Mutex};
//use ctrlc;

fn main() -> std::io::Result<()> {
    // Pass the storage_manager to API modules as needed
    let args: Vec<String> = env::args().collect();

    if args.contains(&"--cli".to_string()) {
        run_cli();
    }

    Ok(())
}
