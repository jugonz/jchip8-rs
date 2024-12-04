/// A trait that describes the visual aspects of an emulated device.
pub trait Drawable {
    fn clear_all_pixels(&mut self);
    fn xor_pixel(&mut self, x: u16, y: u16);

    fn get_pixel(&self, x: u16, y: u16) -> bool;
    /// Determine if a given [x, y] coordinate is
    /// within the bounds of the emulated device.
    /// The `x` and `y` arguments are of type `u32`
    /// for easier numerical manipulation by callers.
    fn in_bounds(&self, x: u32, y: u32) -> bool;
}
