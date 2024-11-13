pub enum SetKeysResult {
    ShouldContinue,
    ShouldExit,
    ShouldSaveState,
}

pub trait Interactible {
    fn set_keys(&mut self) -> SetKeysResult;
    fn get_keys(&self) -> &[bool]; // true if pressed, false otherwise
    fn key_is_pressed(&self, key: u8) -> bool; // true if pressed
}
