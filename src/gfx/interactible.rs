pub trait Interactible {
    fn set_keys(&mut self) -> bool; // false if should exit

    fn is_key_pressed(&self, key: u8) -> bool;
    // fn should_close(&self) -> bool;

    // fn quit(&mut self);
}