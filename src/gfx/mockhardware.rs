#![allow(dead_code)]
use super::interactible::SetKeysResult;
use super::interactible::Interactible;
use super::screen::Screen;

pub struct MockHardware {
    debug: bool,
    title: String,
    keyboard: [bool; 1], // True if a key is pressed.
}

impl MockHardware {
    pub fn new(
        _screen: &Screen,
        debug: bool,
        title: &str,
    ) -> MockHardware {
        MockHardware {
            debug,
            title: String::from(title),
            keyboard: [false; 1],
        }
    }
}

impl Interactible for MockHardware {
    fn init(&mut self) {}

    fn set_title(&mut self, title: &str) -> Result<(), std::io::Error> {
        self.title = String::from(title);
        Ok(())
    }

    fn update_display(&mut self, _screen: &Screen) {}

    fn set_keys(&mut self, _screen: &Screen) -> SetKeysResult {
        SetKeysResult::ShouldContinue
    }

    fn get_keys(&self) -> &[bool] {
        &self.keyboard
    }

    fn key_is_pressed(&self, _key: u8) -> bool {
        false
    }
}

impl Default for MockHardware {
    fn default() -> MockHardware {
        let screen = Screen::default();
        MockHardware::new(&screen, false, "")
    }
}
