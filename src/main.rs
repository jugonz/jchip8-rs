pub mod arch;
pub mod gfx;

use arch::{chip8, Emulator};
use clap::Parser;

#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// Path to the game to load
    #[arg(short, long)]
    path: String,

    /// Path of game state to load
    #[arg(short, long)]
    load_state: Option<String>,

    /// Path of game state to save
    #[arg(short, long)]
    save_state: Option<String>,

    /// Whether or not to turn on debug logging
    #[arg(short, long)]
    debug: bool,
}

fn main() -> Result<(), std::io::Error> {
    let args = Args::parse();

    if let Some(load_state) = args.load_state {
        let mut emulator = chip8::Chip8::from_state(&load_state, args.save_state)?;
        emulator.run();
    } else {
        let mut emulator = chip8::Chip8::new_with_state_path(args.debug, args.save_state);
        emulator.load_game(args.path)?;
        emulator.run();
    }

    Ok(())
}
