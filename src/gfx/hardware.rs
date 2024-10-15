use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

use super::drawable::Drawable;
use super::interactible::Interactible;

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

pub struct Hardware {
    #[allow(unused)]
    width: u32,
    height: u32,
    res_width: u32,
    res_height: u32,
    x_display_scale: u32,
    y_display_scale: u32,
    debug: bool,
    pixels: Vec<Vec<bool>>,
    title: String,
    sdl: sdl2::Sdl,
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    events: Option<sdl2::EventPump>,
    keyboard: [bool; KEYBOARD_LAYOUT.len()], // True if a key is pressed.
}

impl Hardware {
    pub fn new(
        width: u32,
        height: u32,
        res_width: u32,
        res_height: u32,
        debug: bool,
        title: String,
    ) -> Hardware {
        // Check arguments.
        if width == 0 || height == 0 || res_width == 0 || res_height == 0 {
            panic!("Zero screen resolution provided: w{width} h{height} rw{res_width} rh{res_height}");
        }
        let x_display_scale = width / res_width;
        let y_display_scale = height / res_height;
        if x_display_scale == 0 {
            panic!("Invalid screen resolution provided: w{width} does not divide into rw{res_width}");
        } else if y_display_scale == 0 {
            panic!("Invalid screen resolution provided: h{height} does not divide into rh{res_height}");
        }

        // We allow SDL initialization actions to fail with panics
        // as that likely indicates a problem with SDL setup
        // or misuse here!
        let sdl = sdl2::init().unwrap();
        let window = sdl
            .video()
            .unwrap()
            .window(&title, width, height)
            .position_centered()
            .build()
            .unwrap();

        Hardware {
            width,
            height,
            res_width,
            res_height,
            x_display_scale,
            y_display_scale,
            pixels: vec![vec![false; res_height as usize]; res_width as usize],
            debug,
            title,
            sdl,
            canvas: window.into_canvas().build().unwrap(),
            events: None,
            keyboard: [false; 16],
        }
    }

    pub fn set_title(&mut self, title: String) -> Result<(), std::io::Error> {
        self.title = title;

        if let Err(err) = self.canvas.window_mut().set_title(&self.title) {
            Err(std::io::Error::from(err))
        } else {
            Ok(())
        }
    }

    pub fn init(&mut self) {
        // This is a singleton - so we cannot reference the event pump
        // *inside* of self.sdl inside of the constructor, since it's
        // being moved there - it needs to be referenced elsewhere,
        // like here.
        self.events = Some(self.sdl.event_pump().unwrap());
    }

    pub fn handle_pause(&mut self) -> bool {
        // The pause key has been pressed, so we must
        // draw the pause icon on the screen
        // and wait until we either quit or resume.
        if self.debug {
            println!("Pausing!");
        }

        self.draw_pause();

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
        self.draw();
        return true;
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
        return true;
    }

    #[cfg(test)]
    pub fn get_pixels(&self) -> &Vec<Vec<bool>> {
        return &self.pixels;
    }
}

impl Drawable for Hardware {
    fn draw(&mut self) {
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();

        self.canvas.set_draw_color(Color::RGB(255, 255, 255));
        for (xindex, xarr) in self.pixels.iter().enumerate() {
            for (yindex, pixel) in xarr.iter().enumerate() {
                if *pixel {
                    let xcoord = ((xindex as u32) * self.x_display_scale) as i32;
                    let ycoord = ((yindex as u32) * self.y_display_scale) as i32;

                    let rect = Rect::new(xcoord, ycoord, self.x_display_scale, self.y_display_scale);
                    self.canvas.fill_rect(rect).unwrap();
                }
            }
        }

        self.canvas.present();
    }

    fn draw_pause(&mut self) {
        // We want to draw a pause icon in the middle of the screen.
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();

        self.canvas.set_draw_color(Color::RGB(255, 255, 255));
        let xcoord = (self.res_width / 2) - (self.res_width / 12); // roughly lhs of middle of screen
        let ycoord = self.res_height / 3; // roughly top of middle of screen
        let height = self.height / 3;
        if self.in_bounds(xcoord, ycoord) {
            let rect = Rect::new(
                (xcoord * self.x_display_scale) as i32,
                (ycoord * self.y_display_scale) as i32,
                self.x_display_scale,
                height,
            );
            self.canvas.fill_rect(rect).unwrap();
        }

        let xcoord = (self.res_width / 2) + (self.res_width / 12); // roughly rhs of middle of screen
        if self.in_bounds(xcoord, ycoord) {
            let rect = Rect::new(
                (xcoord * self.x_display_scale) as i32,
                (ycoord * self.y_display_scale) as i32,
                self.x_display_scale,
                height,
            );
            self.canvas.fill_rect(rect).unwrap();
        }

        self.canvas.present();
    }

    fn clear_screen(&mut self) {
        self.pixels.iter_mut().for_each(|x| x.fill(false));
    }

    fn xor_pixel(&mut self, x: u16, y: u16) {
        let x_us = usize::from(x);
        let y_us = usize::from(y);
        self.pixels[x_us][y_us] = self.pixels[x_us][y_us] != true;
    }

    fn get_pixel(&self, x: u16, y: u16) -> bool {
        return self.pixels[x as usize][y as usize];
    }

    fn in_bounds(&self, x: u32, y: u32) -> bool {
        return x < self.res_width && y < self.res_height;
    }
}

impl Interactible for Hardware {
    fn set_keys(&mut self) -> bool {
        let Some(event_pump) = &mut self.events else {
            // If the event pump is gone, we're already quitting,
            // so don't process any keys this cycle (and exit!).
            return false;
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

        // Check if we need to pause (and if so, if we quit during the pause).
        if keyboard_state.is_scancode_pressed(KEY_PAUSE) {
            if !self.handle_pause() {
                return false;
            }
        }

        // Check if we need to quit (this is more complicated than
        // just a key being pressed).
        return self.handle_quit();
    }

    fn get_keys(&self) -> &[bool] {
        return &self.keyboard;
    }

    fn key_is_pressed(&self, key: u8) -> bool {
        return self.keyboard[key as usize];
    }
}
