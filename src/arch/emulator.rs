/// A trait that describes a simple emulated device's behavior.
pub trait Emulator {
    fn run(&mut self); // Returns when game or user quits.
}
