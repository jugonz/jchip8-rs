use super::Screen;
use std::io::Error;

/// An enum describing what a caller should do
/// after keyboard input has been processed.
pub enum SetKeysResult {
    // Caller should continue execution.
    ShouldContinue,
    // Caller should exit - a quit key or
    // event that triggers exiting has occurred.
    ShouldExit,
    // Caller should continue execution,
    // but should attempt to save its current
    // state to disk first.
    ShouldSaveState,
}

/// A trait that describes the interactible aspects of an emulated device
/// (screen and keyboard input).
pub trait Interactible {
    fn init(&mut self);
    fn set_title(&mut self, title: &str) -> Result<(), Error>;
    fn update_display(&mut self, screen: &Screen);

    /// Translate keyboard input into action.
    /// This returns an enum that indicates what the caller
    /// should do as a result of that input.
    fn set_keys(&mut self, screen: &Screen) -> SetKeysResult;
    fn get_keys(&self) -> &[bool]; // True if pressed.
    fn key_is_pressed(&self, key: u8) -> bool; // True if pressed.
}
