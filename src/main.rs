//! Program entry point.
//!
//! Handles argument parsing, file loading, config loading, and starts the main editor loop.

use std::{fs, env, io, path::PathBuf};

mod editor;
mod navigation;
mod display;
mod edit;
mod config;
mod search;

use editor::{MicroHex};
use config::AppConfig;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: microhex <file>");
        return Ok(());
    }

    let mut bytes = match fs::read(&args[1]) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", &args[1], e);
            return Ok(());
        }
    };

    if bytes.is_empty() {
        bytes.push(0);
    }

    // Use TOML config file
    let exe_dir: PathBuf = env::current_exe()?.parent().unwrap().to_path_buf();
    let config_path = exe_dir.join("config.toml");
    let config = AppConfig::load(config_path.to_str().unwrap());

    let mut editor = MicroHex::new(args[1].clone(), bytes)?;
    editor.run(&config.colors)?;

    Ok(())
}