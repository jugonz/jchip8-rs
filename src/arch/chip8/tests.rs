use super::*;

#[test]
fn add() {
    let mut c8 = Chip8::new(false);

    c8.opcode = Opcode::new(0x7212);
    c8.decode_exeucte();

    // Test that value is now correct.
    assert_eq!(c8.registers[2], 0x12);

    // Test the max value in a different register.
    c8.opcode = Opcode::new(0x73FF);
    c8.decode_exeucte();

    // Test that value is now correct.
    assert_eq!(c8.registers[3], 0xFF);

    // Now, add one, and test overflow.
    c8.opcode = Opcode::new(0x7301);
    c8.decode_exeucte();

    // Test that value is now correct.
    assert_eq!(c8.registers[3], 0x00);
}

#[test]
fn add_with_carry() {
    let mut c8 = Chip8::new(true);

    // Test adding the max value without overflow.
    c8.opcode = Opcode::new(0x73FF); // Add FF to reg 3 (0).
    c8.decode_exeucte();
    c8.opcode = Opcode::new(0x8374); // Add reg 7 (0) to reg 3.
    c8.decode_exeucte();

    // Test that value is now correct.
    assert_eq!(c8.registers[3], 0xFF);
    assert_eq!(c8.registers[0xF], 0);

    // Now, add one, and test overflow.
    c8.opcode = Opcode::new(0x7401); // Add 1 to reg 4 (0).
    c8.decode_exeucte();
    c8.opcode = Opcode::new(0x8344); // Add reg 4 (1) to reg 3 (FF).
    c8.decode_exeucte();

    // Test that value is now correct.
    assert_eq!(c8.registers[3], 0);
    // Test that the overflow register is correctly set.
    assert_eq!(c8.registers[0xF], 1);
}

#[test]
fn call_return() {
    let mut c8 = Chip8::new(false);

    // Make sure the stack is initially empty.
    assert_eq!(c8.sp, 0);
    for val in c8.stack {
        assert_eq!(val, 0, "Found non-zero value in stack");
    }

    // Call a program at 789 and check the stack.
    c8.opcode = Opcode::new(0x2789);
    c8.decode_exeucte();

    assert_eq!(c8.sp, 1);
    assert_eq!(c8.stack[0], 0x200);

    for val in c8.stack[1..].iter() {
        assert_eq!(*val, 0, "Found non-zero value in upper part of stack");
    }

    // Return from that program and make sure
    // variables are reset.
    c8.opcode = Opcode::new(0x00EE);
    c8.decode_exeucte();

    assert_eq!(c8.sp, 0);
    for val in c8.stack[1..].iter() {
        assert_eq!(*val, 0, "Found non-zero value in stack");
    }
}
