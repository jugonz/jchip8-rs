pub trait Emulator {
    fn load_game(&mut self, file_path: String) -> Result<(), std::io::Error>;
    fn test_init(&mut self); // FOR SCREEN TESTING
    fn run(&mut self); // Returns when game or user quits.
}