#![allow(dead_code)]
use super::interactible::SetKeysResult;
use super::interactible::Interactible;
use super::screen::Screen;

pub struct MockHardware {
    debug: bool,
    title: String,
    keyboard: [bool; 16], // True if a key is pressed.
}

impl MockHardware {
    pub fn new(
        _screen: &Screen,
        debug: bool,
        title: String,
    ) -> MockHardware {
        MockHardware {
            debug,
            title,
            keyboard: [false; 16],
        }
    }

    pub fn init(&mut self) {}
}

impl Interactible for MockHardware {
    fn init(&mut self) {}

    fn set_title(&mut self, title: String) -> Result<(), std::io::Error> {
        self.title = title;

        Ok(())
    }

    fn update_display(&mut self, _screen: &Screen) {}

    fn set_keys(&mut self, _screen: &Screen) -> SetKeysResult {
        SetKeysResult::ShouldContinue
    }

    fn get_keys(&self) -> &[bool] {
        return &self.keyboard;
    }

    fn key_is_pressed(&self, _key: u8) -> bool {
        return false;
    }
}

impl Default for MockHardware {
    fn default() -> MockHardware {
        let screen = Screen::new(640, 480, 64, 32);
        MockHardware::new(&screen, false, String::from(""))
    }
}
