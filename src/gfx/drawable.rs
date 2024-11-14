// use super::screen::Screen;

pub trait Drawable {
    fn clear_all_pixels(&mut self);
    fn xor_pixel(&mut self, x: u16, y: u16);

    fn get_pixel(&self, x: u16, y: u16) -> bool;

    fn get_pixels(&self) -> &Vec<Vec<bool>>;

    fn in_bounds(&self, x: u32, y: u32) -> bool;
}
