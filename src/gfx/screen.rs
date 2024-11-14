use serde::{Serialize, Deserialize};
use serde_with::serde_as;

use super::Drawable;

#[serde_as]
#[derive(Serialize, Deserialize)]
#[derive(Default)]
pub struct Screen {
    pub width: u32,
    pub height: u32,
    pub res_width: u32,
    pub res_height: u32,
    pub x_display_scale: u32,
    pub y_display_scale: u32,
    pixels: Vec<Vec<bool>>,
}

impl Screen {
    pub fn new(
        width: u32,
        height: u32,
        res_width: u32,
        res_height: u32,
    ) -> Screen {
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

        Screen {
            width,
            height,
            res_width,
            res_height,
            x_display_scale,
            y_display_scale,
            pixels: vec![vec![false; res_height as usize]; res_width as usize],
        }
    }
}

impl Drawable for Screen {
    // Setters
    fn clear_all_pixels(&mut self) {
        self.pixels.iter_mut().for_each(|x| x.fill(false));
    }

    fn xor_pixel(&mut self, x: u16, y: u16) {
        let x_us = x as usize;
        let y_us = y as usize;
        self.pixels[x_us][y_us] = self.pixels[x_us][y_us] != true;
    }

    // Getters
    fn get_pixel(&self, x: u16, y: u16) -> bool {
        return self.pixels[x as usize][y as usize];
    }

    fn get_pixels(&self) -> &Vec<Vec<bool>> {
        return &self.pixels;
    }

    // Info
    fn in_bounds(&self, x: u32, y: u32) -> bool {
        return x < self.res_width && y < self.res_height;
    }
}

// impl Iterator for Screen {
//     type Item = (usize, usize);

//     fn next(&mut self) -> Option<(usize, usize)> {
//         for (xindex, xarr) in self.pixels.iter().enumerate() {
//             for (yindex, pixel) in xarr.iter().enumerate() {
//                 if *pixel {
//                     return Some((xindex, yindex))
//                 }
//             }
//         }

//         None
//     }
// }