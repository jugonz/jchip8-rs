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
const KEY_QUIT : Scancode = Scancode::Escape;

pub struct Hardware {
    width: u32,
    height: u32,
    res_width: u32,
    res_height: u32,
    debug: bool,
    pixels: Vec<Vec<bool>>,
    title: String,
    sdl: Option<sdl2::Sdl>,
    canvas: Option<sdl2::render::Canvas<sdl2::video::Window>>,
    events: Option<sdl2::EventPump>,
    keyboard: [bool; KEYBOARD_LAYOUT.len()], // True if a key is pressed.
}

impl Hardware {
    pub fn new(width: u32, height: u32, res_width: u32, res_height: u32, debug: bool, title: String) -> Hardware {
        Hardware {
            width,
            height,
            res_width,
            res_height,
            pixels: vec![vec![false; res_height as usize]; res_width as usize],
            debug,
            title,
            sdl: None,
            canvas: None,
            events: None,
            keyboard: [false; 16],
        }
    }

    pub fn set_title(&mut self, title: String) {
        self.title = title
    }

    pub fn init(&mut self) {
        let sdl_context = sdl2::init().unwrap();
        let video_sbsys = sdl_context.video().unwrap();

        let window = video_sbsys
            .window(&self.title, self.width, self.height)
            .position_centered()
            .build()
            .unwrap();
        let mut canvas = window.into_canvas().build().unwrap();
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        self.events = Some(sdl_context.event_pump().unwrap());
        self.sdl = Some(sdl_context);
        self.canvas = Some(canvas);
    }

    #[cfg(test)]
    pub fn get_pixels(&self) -> &Vec<Vec<bool>> {
        return &self.pixels;
    }
}

impl Drawable for Hardware {
    fn draw(&mut self) {
        // Return early if the canvas is gone.
        let Some(canvas) = self.canvas.as_mut() else { return };

        let display_scale = std::cmp::min(self.width / self.res_width, self.height / self.res_height);

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        canvas.set_draw_color(Color::RGB(255, 255, 255));
        for (xindex, xarr) in self.pixels.iter().enumerate() {
            for (yindex, pixel) in xarr.iter().enumerate() {
                if *pixel {
                    let xcoord = ((xindex as u32) * display_scale) as i32;
                    let ycoord = ((yindex as u32) * display_scale) as i32;

                    let rect = Rect::new(xcoord, ycoord, display_scale, display_scale);
                    canvas.fill_rect(rect).unwrap();
                }
            }
        }

        canvas.present();
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

    fn in_bounds(&self, x: u16, y: u16) -> bool {
        return (u32::from(x)) < self.res_width && (u32::from(y) as u32) < self.res_height;
    }
}

impl Interactible for Hardware {
    fn set_keys(&mut self) -> bool {
        let Some(event_pump) = &mut self.events else {
            // If the event pump is gone, we're already quitting,
            // so don't process any keys this cycle (and exit!).
            return false
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

        // Check if we need to quit.
        // This can happen via either (a) the quit key being pressed
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

        return true;
    }

    fn get_keys(&self) -> &[bool] {
        return &self.keyboard;
    }

    fn key_is_pressed(&self, key: u8) -> bool {
        return self.keyboard[key as usize];
    }
}
