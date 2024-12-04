/// A trait that describes the operations of the CPU of an emulated device.
pub trait InstructionSet {
    // Graphics controls.
    fn clear_screen(&mut self);
    fn draw_sprite(&mut self);
    fn set_index_reg_to_sprite(&mut self);

    // Control flow.
    fn call(&mut self);
    fn r#return(&mut self);
    fn jump(&mut self);
    fn jump_with_offset(&mut self);
    fn skip_if_eq_literal(&mut self);
    fn skip_if_not_eq_literal(&mut self);
    fn skip_if_eq_reg(&mut self);
    fn skip_if_not_eq_reg(&mut self);
    fn skip_if_key_pressed(&mut self);
    fn skip_if_key_not_pressed(&mut self);

    // Manipulating data registers.
    fn set_reg_to_literal(&mut self);
    fn set_reg_to_reg(&mut self);
    fn add(&mut self);
    fn add_with_carry(&mut self);
    fn or(&mut self);
    fn and(&mut self);
    fn xor(&mut self);
    // Subtract the lower register from the higher register.
    fn sub_x_from_y(&mut self);
    // Subtract the higher register from the lower register.
    fn sub_y_from_x(&mut self);
    fn shift_right(&mut self);
    fn shift_left(&mut self);
    fn set_reg_random_mask(&mut self);
    fn save_binary_coded_decimal(&mut self);

    // Manipulating special registers.
    fn add_reg_to_index_reg(&mut self);
    fn set_index_reg_to_literal(&mut self);
    fn get_key_press(&mut self);
    fn get_delay_timer(&mut self);
    fn set_delay_timer(&mut self);
    fn set_sound_timer(&mut self);

    // Context switching.
    fn save_registers(&mut self);
    fn restore_registers(&mut self);

    // Save state handling.
    fn save_state(&mut self);
}
