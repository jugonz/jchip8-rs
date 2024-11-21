use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

use super::Drawable;
use super::interactible::SetKeysResult;
use super::interactible::Interactible;
use super::screen::Screen;

const KEYBOARD_LAYOUT: [Scancode; 16] = [
    Scancode::Num0,
    Scancode::Num1,
    Scancode::Num2,
    Scancode::Num3,
    Scancode::Num4,
    Scancode::Num5,
    Scancode::Num6,
    Scancode::Num7,
    Scancode::Num8,
    Scancode::Num9,
    Scancode::A,
    Scancode::B,
    Scancode::C,
    Scancode::D,
    Scancode::E,
    Scancode::F,
];
const KEY_QUIT: Scancode = Scancode::Escape;
const KEY_PAUSE: Scancode = Scancode::P;
const KEY_SAVE_STATE: Scancode = Scancode::S;
const NO_GAME_LOADED: &str = "No game loaded";

pub struct Hardware {
    debug: bool,
    title: String,
    sdl: sdl2::Sdl,
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    events: Option<sdl2::EventPump>,
    keyboard: [bool; KEYBOARD_LAYOUT.len()], // True if a key is pressed.
}

impl Hardware {
    pub fn new(
        screen: &Screen,
        debug: bool,
        title: String,
    ) -> Hardware {
        // We allow SDL initialization actions to fail with panics
        // as that likely indicates a problem with SDL setup
        // or misuse here!
        let sdl = sdl2::init().unwrap();
        let window = sdl
            .video()
            .unwrap()
            .window(&title, screen.width, screen.height)
            .position_centered()
            .build()
            .unwrap();

        Hardware {
            debug,
            title,
            sdl,
            canvas: window.into_canvas().build().unwrap(),
            events: None,
            keyboard: [false; KEYBOARD_LAYOUT.len()],
        }
    }

    fn handle_pause(&mut self, screen: &Screen) -> bool {
        // The pause key has been pressed, so we must
        // draw the pause icon on the screen
        // and wait until we either quit or resume.
        if self.debug {
            println!("Pausing!");
        }

        self.draw_pause(screen);

        let Some(event_pump) = &mut self.events else {
            // If the event pump is gone, we're already quitting,
            // so don't process any keys this cycle (and exit!).
            return false;
        };

        // Sit on the event pump until:
        // (a) We need to quit.
        //   Like in handle_quit(), this can happen if:
        //     (1) quit is pressed
        //     (2) we receive the Quit event
        // (b) We get the pause event again (unpause).
        //
        //   This is slightly complicated - we don't want to accept
        //   an unpause event until the pause key is first released
        //   so given that we're in this block because the key was pressed,
        //   we first wait for a KeyUp event for the pause key.
        //   Once that's delivered, a KeyDown event followed by a KeyUp event
        //   for the pause key will unpause the emulation.
        //   Note that we can still quit while this is all happening.
        let mut key_raised = false;
        let mut key_released = false;
        for event in event_pump.wait_iter() {
            match event {
                // (a)
                Event::Quit { .. }
                | Event::KeyDown {
                    scancode: Some(KEY_QUIT),
                    ..
                } => {
                    if self.debug {
                        println!("Quitting!");
                    }
                    return false;
                }
                Event::KeyDown {
                    scancode: Some(KEY_PAUSE),
                    ..
                } if key_released => {
                    if self.debug {
                        println!("Saw Pause Keydown!");
                    }
                    key_raised = true;
                }
                Event::KeyUp {
                    scancode: Some(KEY_PAUSE),
                    ..
                } => {
                    // (b)
                    if key_raised {
                        if self.debug {
                            println!("Unpausing!");
                        }
                        // Clear "Pause" icon here.

                        break;
                    } else {
                        if self.debug {
                            println!("First key up!");
                        }

                        // Else, this is the key up from the actual pause press.
                        key_released = true;
                    }
                }
                _ => (),
            }
        }

        // We've unpaused, so it's time to re-draw the screen and resume.
        self.update_display(&screen);
        true
    }

    pub fn handle_quit(&mut self) -> bool {
        // Check for quit (note that unlike handle_pause()
        // the quit key has not necessarily been pressed).

        let Some(event_pump) = &mut self.events else {
            // If the event pump is gone, we're quitting by definition.
            return false;
        };
        let keyboard_state = event_pump.keyboard_state();

        // Quitting can happen via either (a) the quit key being pressed
        // or (b) the SDL quit event being sent through the event pump.

        // (a)
        if keyboard_state.is_scancode_pressed(KEY_QUIT) {
            if self.debug {
                println!("Quitting due to escape key!");
            }
            return false;
        }

        // (b)
        for event in event_pump.poll_iter() {
            if let Event::Quit { .. } = event {
                if self.debug {
                    println!("Quitting!");
                }
                return false;
            }
        }

        // We're not quitting.
        true
    }

    fn draw_pause(&mut self, screen: &Screen) {
        // We want to draw a pause icon in the middle of the screen.
        self.canvas.set_draw_color(Color::BLACK);
        self.canvas.clear();

        self.canvas.set_draw_color(Color::WHITE);
        let xcoord = (screen.res_width / 2) - (screen.res_width / 12); // Roughly lhs of middle of screen.
        let ycoord = screen.res_height / 3; // Roughly top of middle of screen.
        let height = screen.height / 3;
        // If the coordinate is in bounds, it should be safe to convert to / from 32-bit types.
        if screen.in_bounds(xcoord, ycoord) {
            let rect = Rect::new(
                (xcoord * screen.x_display_scale) as i32,
                (ycoord * screen.y_display_scale) as i32,
                screen.x_display_scale,
                height,
            );
            self.canvas.fill_rect(rect).unwrap();
        }

        let xcoord = (screen.res_width / 2) + (screen.res_width / 12); // Roughly rhs of middle of screen.
        if screen.in_bounds(xcoord, ycoord) {
            let rect = Rect::new(
                (xcoord * screen.x_display_scale) as i32,
                (ycoord * screen.y_display_scale) as i32,
                screen.x_display_scale,
                height, // Same height as other drawn rectangle.
            );
            self.canvas.fill_rect(rect).unwrap();
        }

        self.canvas.present();
    }
}

impl Interactible for Hardware {
    fn init(&mut self) {
        // This is a singleton - so we cannot reference the event pump
        // *inside* of self.sdl inside of the constructor, since it's
        // being moved there - it needs to be referenced elsewhere,
        // like here.
        self.events = Some(self.sdl.event_pump().unwrap());
    }

    fn set_title(&mut self, title: String) -> Result<(), std::io::Error> {
        self.title = title;

        if let Err(err) = self.canvas.window_mut().set_title(&self.title) {
            Err(std::io::Error::from(err))
        } else {
            Ok(())
        }
    }

    fn update_display(&mut self, screen: &Screen) {
        // First, re-draw the entire canvas with black.
        self.canvas.set_draw_color(Color::BLACK);
        self.canvas.clear();

        // Next, draw the set pixels with white.
        self.canvas.set_draw_color(Color::WHITE);

        let pixels = screen.get_pixels();
        for (xindex, xarr) in pixels.iter().enumerate() {
            for (yindex, pixel) in xarr.iter().enumerate() {
                if *pixel {
                    // Since these indices are from our vector,
                    // they should be safe to convert to / from 32-bit types.
                    let xcoord = ((xindex as u32) * screen.x_display_scale) as i32;
                    let ycoord = ((yindex as u32) * screen.y_display_scale) as i32;

                    let rect = Rect::new(xcoord, ycoord, screen.x_display_scale, screen.y_display_scale);
                    self.canvas.fill_rect(rect).unwrap();
                }
            }
        }

        self.canvas.present();
    }

    fn set_keys(&mut self, screen: &Screen) -> SetKeysResult {
        let Some(event_pump) = &mut self.events else {
            // If the event pump is gone, we're already quitting,
            // so don't process any keys this cycle (and exit!).
            return SetKeysResult::ShouldExit;
        };

        // Check for keyboard input.
        let keyboard_state = event_pump.keyboard_state();
        for (index, key) in KEYBOARD_LAYOUT.iter().enumerate() {
            if keyboard_state.is_scancode_pressed(*key) {
                if self.debug {
                    println!("{} was pressed!", *key);
                }
                self.keyboard[index] = true;
            } else {
                self.keyboard[index] = false;
            }
        }

        // Now that keys have been processed,
        // check what action we will return to our caller.
        //
        // There are three actions:
        // If a quit key has been pressed, we will ask our caller
        // to start exiting. Otherwise, our caller will continue
        // executing - but it may be asked to save its state out to disk.
        //
        // Processing this is slightly complicated, as we also
        // handle pausing from this code as well first.
        // We are guaranteed that when we run we are not paused,
        // and we guarantee that we return unpaused - so
        // any waiting is handled by pause-code invoked here.
        // Note that quitting while paused is also supported,
        // so both methods may instruct us to quit.
        //
        // Since pause-handling and quit-handling code both take control
        // of the event pump, we must check if a save state was requested
        // before we handle pausing (we don't support saving states during pauses).
        // If either pause-handling code and quit-handling code
        // told us to quit, we won't act on the save state request.
        //
        // Finally, it's important that quit-handling code runs last,
        // as it cycles through the event pump and ensures we don't get
        // duplicate results about any key presses (including pause / save state)
        // the next time we're here.
        let mut caller_action = SetKeysResult::ShouldContinue;

        // Check if the save state key was pressed.
        // If so, we'll return to our caller that it was pressed
        // *only* if we're not pausing or quitting.
        if keyboard_state.is_scancode_pressed(KEY_SAVE_STATE) {
            if self.debug {
                println!("Saving state!");
            }
            caller_action = SetKeysResult::ShouldSaveState;
        }

        // Check if we need to pause (and if so, if we quit during the pause).
        // (We don't allow saving states while paused, so we'll ignore
        // any key presses above for saving states.)
        if keyboard_state.is_scancode_pressed(KEY_PAUSE) && !self.handle_pause(&screen) {
            return SetKeysResult::ShouldExit;
        }

        // Check if we need to quit - if not,
        // we'll continue (and save state if we saw the key press above).
        match self.handle_quit() {
            false => SetKeysResult::ShouldExit,
            true => caller_action,
        }
    }

    fn get_keys(&self) -> &[bool] {
        &self.keyboard
    }

    fn key_is_pressed(&self, key: u8) -> bool {
        self.keyboard[key as usize]
    }
}

impl Default for Hardware {
    fn default() -> Hardware {
        let screen = Screen::default();
        Hardware::new(&screen, false, String::from(NO_GAME_LOADED))
    }
}
