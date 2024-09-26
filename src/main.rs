use arch::{Emulator, chip8};

pub mod arch;
pub mod gfx;

fn main() {
    let mut emulator = chip8::Chip8::new(true);
    emulator.test_init();
}
