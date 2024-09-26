pub trait InstructionSet {
    fn call(&mut self);
    fn r#return(&mut self);
    fn jump(&mut self);

    // Manipulating data registers
    fn set_reg_to_literal(&mut self);
    fn set_reg_to_reg(&mut self);

    fn add(&mut self);
    fn add_with_carry(&mut self);
    fn or(&mut self);
    fn and(&mut self);
    fn xor(&mut self);
    fn sub_x_from_y(&mut self);
    fn sub_y_from_x(&mut self);
    fn shift_right(&mut self);
    fn shift_left(&mut self);
}