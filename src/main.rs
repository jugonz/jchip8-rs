use arch::{Emulator, chip8};

pub mod arch;
pub mod gfx;

fn main() {
    let mut emulator = chip8::Chip8::new(true);
    emulator.load_game(String::from("c8games/PONG")).unwrap();
    emulator.run();


    // emulator.test_init();
}
