pub trait Emulator {
    fn run(&mut self); // Returns when game or user quits.
}
