use std::{fs, env, io};

mod editor;
mod navigation;
mod display;
mod edit;

use editor::{MicroHex};

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

    let mut editor = MicroHex::new(args[1].clone(), bytes)?;
    editor.run()?;

    Ok(())
}