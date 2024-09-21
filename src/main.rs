#![allow(dead_code)]

use crate::gfx::Interactible;
extern crate sdl2;

pub mod arch;
pub mod gfx;

fn main() {
    let mut screen = gfx::Screen::new(300, 300, 64, 32, String::from("Chip-8 Emulator"));
    screen.init();

    while screen.set_keys() {}
}
