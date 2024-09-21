#![allow(dead_code)]
#![allow(unused_imports)]

use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::pixels::Color;
use sdl2::rect::Rect;

use std::time::Duration;

extern crate sdl2;

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
const DISPLAY_SCALE: u32 = 15; // TODO: assert that all pixels fit

pub trait Drawable {
    fn draw(&mut self);
    fn clear_screen(&mut self);
    fn xor_pixel(&mut self, x: u16, y: u16);

    fn get_pixel(&self, x: u16, y: u16) -> bool;
    fn in_bounds(&self, x: u16, y: u16) -> bool;
}

pub trait Interactible {
    fn set_keys(&mut self) -> bool; // false if should exit

    fn is_key_pressed(&self, key: u8) -> bool;
    // fn should_close(&self) -> bool;

    // fn quit(&mut self);
}

pub struct Screen {
    width: u32,
    height: u32,
    res_width: u32,
    res_height: u32,
    pixels: Vec<Vec<bool>>,
    title: String,
    sdl: Option<sdl2::Sdl>,
    canvas: Option<sdl2::render::Canvas<sdl2::video::Window>>,
    events: Option<sdl2::EventPump>,
    keyboard: [bool; 16], // True if a key is pressed.
}

impl Screen {
    pub fn new(width: u32, height: u32, res_width: u32, res_height: u32, title: String) -> Screen {
        Screen {
            width,
            height,
            res_width,
            res_height,
            pixels: vec![vec![false; res_height as usize]; res_width as usize],
            title,
            sdl: None,
            canvas: None,
            events: None,
            keyboard: [false; 16],
        }
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

        // let mut i = 0;
        // 'running: loop {
        //     i = (i + 1) % 255;
        //     canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
        //     canvas.clear();
        //     for event in events.poll_iter() {
        //         match event {
        //             Event::Quit {..} |
        //             Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
        //                 break 'running
        //             },
        //             _ => {}
        //         }
        //     }
        //     // The rest of the game loop goes here...

        //     canvas.present();
        //     ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        // }

        self.sdl = Some(sdl_context); // TODO: At end!
                                      // self.window = Some(window);
        self.canvas = Some(canvas);
    }
}

impl Drawable for Screen {
    fn draw(&mut self) {
        let canvas = self.canvas.as_mut().unwrap();
        canvas.set_draw_color(Color::RGB(0, 0, 0));

        for (xindex, xarr) in self.pixels.iter().enumerate() {
            for (yindex, pixel) in xarr.iter().enumerate() {
                if *pixel {
                    // TODO: check if i32::from() is better
                    let xcoord = ((xindex as u32) * DISPLAY_SCALE) as i32;
                    let ycoord = ((yindex as u32) * DISPLAY_SCALE) as i32;

                    let rect = Rect::new(xcoord, ycoord, DISPLAY_SCALE, DISPLAY_SCALE);
                    canvas.fill_rect(rect).unwrap();
                }
            }
        }

        canvas.present();
    }

    fn clear_screen(&mut self) {
        for x in self.pixels.iter_mut() {
            for y in x.iter_mut() {
                *y = false;
            }
        }
    }

    fn xor_pixel(&mut self, x: u16, y: u16) {
        self.pixels[x as usize][y as usize] = self.pixels[x as usize][y as usize] != true;
    }

    fn get_pixel(&self, x: u16, y: u16) -> bool {
        return self.pixels[x as usize][y as usize];
    }

    fn in_bounds(&self, x: u16, y: u16) -> bool {
        return (x as u32) < self.res_width && (y as u32) < self.res_height;
    }
}

impl Interactible for Screen {
    fn set_keys(&mut self) -> bool {
        for (index, key) in KEYBOARD_LAYOUT.into_iter().enumerate() {
            if self
                .events
                .as_mut()
                .unwrap()
                .keyboard_state()
                .is_scancode_pressed(key)
            {
                println!("{} was pressed!\n", key);
                self.keyboard[index] = true;
            } else {
                self.keyboard[index] = false;
            }
        }

        // Seems like QUIT only comes in as an event from the event pump.
        // (If we wanted to quit upon a key press, we'd have to add a check for it around here.)
        for event in self.events.as_mut().unwrap().poll_iter() {
            if let Event::Quit { .. } = event {
                println!("Quitting!\n");
                return false;
            }
        }

        return true;
    }

    fn is_key_pressed(&self, key: u8) -> bool {
        return self.keyboard[key as usize];
    }

    // fn should_close(&self) -> bool {
    //     return false;
    // }

    // fn quit(&mut self) {

    // }
}
