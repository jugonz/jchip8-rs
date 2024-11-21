use super::Screen;
use std::io::Error;

pub enum SetKeysResult {
    ShouldContinue,
    ShouldExit,
    ShouldSaveState,
}

pub trait Interactible {
    fn init(&mut self);
    fn set_title(&mut self, title: &str) -> Result<(), Error>;
    fn update_display(&mut self, screen: &Screen);

    fn set_keys(&mut self, screen: &Screen) -> SetKeysResult;
    fn get_keys(&self) -> &[bool]; // true if pressed, false otherwise
    fn key_is_pressed(&self, key: u8) -> bool; // true if pressed
}
