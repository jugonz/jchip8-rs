use super::interactible::{Interactible, SetKeysResult};
use super::screen::Screen;

#[derive(Default)]
/// A placeholder struct for Hardware that is useful during testing
/// when we cannot call any SDL methods (since our test runner
/// may not run our tests on the main thread, which SDL strictly requires).
pub struct MockHardware {
    pub debug: bool,
    keyboard: [bool; 1],
}

impl MockHardware {
    pub fn new(_screen: &Screen, debug: bool, _title: &str) -> MockHardware {
        MockHardware {
            debug,
            keyboard: [false; 1],
        }
    }
}

impl Interactible for MockHardware {
    fn init(&mut self) {}

    fn set_title(&mut self, _title: &str) -> Result<(), std::io::Error> {
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
