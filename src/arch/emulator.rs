pub trait Emulator {
    fn load_game(&mut self, file_path: String) -> Result<(), std::io::Error>;
    fn run(&mut self); // Returns when game or user quits.
}
