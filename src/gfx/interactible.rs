pub trait Interactible {
    fn set_keys(&mut self) -> bool; // false if should exit
    fn get_keys(&self) -> &[bool]; // true if pressed, false otherwise
    fn key_is_pressed(&self, key: u8) -> bool; // true if pressed
}
