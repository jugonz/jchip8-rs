pub trait Drawable {
    fn draw(&mut self);
    fn draw_pause(&mut self);
    fn clear_screen(&mut self);
    fn xor_pixel(&mut self, x: u16, y: u16);

    fn get_pixel(&self, x: u16, y: u16) -> bool;
    fn in_bounds(&self, x: u32, y: u32) -> bool;
}