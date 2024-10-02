pub mod arch;
pub mod gfx;

use arch::{Emulator, chip8};
use clap::Parser;

#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// Path to the game to load
    #[arg(short, long)]
    path: String,

    /// Whether or not to turn on debug logging
    #[arg(short, long)]
    debug: bool,
}

fn main() -> Result<(), std::io::Error> {
    let args = Args::parse();

    let mut emulator = chip8::Chip8::new(args.debug);
    emulator.load_game(args.path)?;
    emulator.run();

    Ok(())
}
