use super::Drawable;
use std::fmt;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[serde_as]
#[derive(Serialize, Deserialize)]
/// A struct describing a displayable two-dimensional device
/// with individual pixels that are either on or off.
/// It can be queried by pixel or iterated over
/// but can only modified via specific methods.
pub struct Screen {
    // The display width of the device.
    pub width: u32,
    // The display height of the device.
    pub height: u32,
    // The actual width resolution of the device
    // (this will expand to the display width).
    pub res_width: u32,
    // The actual height resolution of the device.
    pub res_height: u32,
    // The ratio width / res_width.
    pub x_display_scale: u32,
    // The ratio height / res_height.
    pub y_display_scale: u32,
    // The actual pixel values.
    pixels: Vec<Vec<bool>>,
}

/// Iterator for a Screen that only returns pixels that are set.
pub struct ScreenIterator<'a> {
    screen: &'a Screen,
    // Keep track of the last (X, Y) pixel we saw that was set.
    curr: (usize, usize),
}

impl Iterator for ScreenIterator<'_> {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        // Iterate only the vectors starting with our current X coordinate.
        for (xindex, xarr) in self.screen.pixels[self.curr.0..].iter().enumerate() {
            // Iterate all Y values, even those we've already seen in the current X vector
            // (we do this to keep updating our saved Y simple).
            for (yindex, pixel) in xarr.iter().enumerate() {
                // Since we sliced above, xindex is the start from the slice, not the entire vector.
                let real_xindex = xindex + self.curr.0;

                if *pixel && ((xindex > 0) || (xindex == 0 && yindex > self.curr.1)) {
                    // If we see a pixel past the last Y we saw in the first vector,
                    // or a pixel in *ANY* vector past the first, it's new. Save it and return it.
                    self.curr = (real_xindex, yindex);
                    return Some(self.curr);
                }
            }
        }

        None
    }
}

// Allow converting references of Screens to iterators
// for easy for loop iteration (but without consuming the Screen object itself).
impl<'a> IntoIterator for &'a Screen {
    type Item = (usize, usize);
    type IntoIter = ScreenIterator<'a>;

    fn into_iter(self) -> ScreenIterator<'a> {
        ScreenIterator {
            screen: self,
            curr: (0, 0),
        }
    }
}

impl Screen {
    pub fn new(width: u32, height: u32, res_width: u32, res_height: u32) -> Screen {
        // Check arguments.
        if width == 0 || height == 0 || res_width == 0 || res_height == 0 {
            panic!(
                "Zero screen resolution provided: w{width} h{height} rw{res_width} rh{res_height}"
            );
        }
        let x_display_scale = width / res_width;
        let y_display_scale = height / res_height;
        if x_display_scale == 0 {
            panic!(
                "Invalid screen resolution provided: w{width} does not divide into rw{res_width}"
            );
        } else if y_display_scale == 0 {
            panic!(
                "Invalid screen resolution provided: h{height} does not divide into rh{res_height}"
            );
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
    // Setters.
    fn clear_all_pixels(&mut self) {
        self.pixels.iter_mut().for_each(|x| x.fill(false));
    }

    fn xor_pixel(&mut self, x: u16, y: u16) {
        let x_us = x as usize;
        let y_us = y as usize;
        self.pixels[x_us][y_us] = self.pixels[x_us][y_us] != true;
    }

    // Getters.
    fn get_pixel(&self, x: u16, y: u16) -> bool {
        self.pixels[x as usize][y as usize]
    }

    // Info.
    fn in_bounds(&self, x: u32, y: u32) -> bool {
        x < self.res_width && y < self.res_height
    }
}

// Mostly useful for debugging.
impl fmt::Display for Screen {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "W: {} H: {} SW: {} SH: {} XS: {} YS: {}",
            self.width,
            self.height,
            self.res_width,
            self.res_height,
            self.x_display_scale,
            self.y_display_scale,
        )
    }
}

impl Default for Screen {
    fn default() -> Screen {
        // For now, our default window size.
        Screen::new(640, 480, 64, 32)
    }
}
