use super::*;

fn run_opcode(c8: &mut Chip8, instruction: u16) {
    c8.opcode = Opcode::new(instruction);
    c8.decode_execute();
}

#[test]
fn test_skip_instruction() {
    let mut c8 = Chip8::new(false);

    // First, add the literal (A3) to a register.
    c8.opcode = Opcode::new(0x71A3);
    c8.decode_execute();
    c8.increment_pc();

    // Now, check the PC.
    assert_eq!(c8.pc, 0x202);

    // Now, check that an instruction is skipped when
    // comparing the literal.
    c8.opcode = Opcode::new(0x31A3);
    c8.decode_execute();
    c8.increment_pc();

    assert_eq!(c8.pc, 0x206);

    // Now, check that an instruction is NOT skipped
    // when comparing the same literal.
    c8.opcode = Opcode::new(0x41A3);
    c8.decode_execute();
    c8.increment_pc();

    assert_eq!(c8.pc, 0x208);

    // Now, check that comparing two identical registers
    // leads to an instruction skip.
    c8.opcode = Opcode::new(0x72A3); // Add literal to another register.
    c8.decode_execute();
    c8.increment_pc();

    c8.opcode = Opcode::new(0x5120);
    c8.decode_execute();
    c8.increment_pc();

    assert_eq!(c8.pc, 0x20E);
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
    c8.decode_execute();

    assert_eq!(c8.sp, 1);
    assert_eq!(c8.stack[0], 0x200);

    for val in c8.stack[1..].iter() {
        assert_eq!(*val, 0, "Found non-zero value in upper part of stack");
    }

    // Return from that program and make sure
    // variables are reset.
    c8.opcode = Opcode::new(0x00EE);
    c8.decode_execute();

    assert_eq!(c8.sp, 0);
    for val in c8.stack[1..].iter() {
        assert_eq!(*val, 0, "Found non-zero value in stack");
    }
}

#[test]
fn add() {
    let mut c8 = Chip8::new(false);

    c8.opcode = Opcode::new(0x7212);
    c8.decode_execute();

    // Test that value is now correct.
    assert_eq!(c8.registers[2], 0x12);

    // Test the max value in a different register.
    c8.opcode = Opcode::new(0x73FF);
    c8.decode_execute();

    // Test that value is now correct.
    assert_eq!(c8.registers[3], 0xFF);

    // Now, add one, and test overflow.
    c8.opcode = Opcode::new(0x7301);
    c8.decode_execute();

    // Test that value is now correct.
    assert_eq!(c8.registers[3], 0x00);
}

#[test]
fn add_with_carry() {
    let mut c8 = Chip8::new(true);

    // Test adding the max value without overflow.
    c8.opcode = Opcode::new(0x73FF); // Add FF to reg 3 (0).
    c8.decode_execute();
    c8.opcode = Opcode::new(0x8374); // Add reg 7 (0) to reg 3.
    c8.decode_execute();

    // Test that value is now correct.
    assert_eq!(c8.registers[3], 0xFF);
    assert_eq!(c8.registers[0xF], 0);

    // Now, add one, and test overflow.
    c8.opcode = Opcode::new(0x7401); // Add 1 to reg 4 (0).
    c8.decode_execute();
    c8.opcode = Opcode::new(0x8344); // Add reg 4 (1) to reg 3 (FF).
    c8.decode_execute();

    // Test that value is now correct.
    assert_eq!(c8.registers[3], 0);
    // Test that the overflow register is correctly set.
    assert_eq!(c8.registers[0xF], 1);
}

#[test]
fn sub() {
    let mut c8 = Chip8::new(true);

    c8.opcode = Opcode::new(0x71A2); // Add A2 to reg 1 (0).
    c8.decode_execute();
    c8.opcode = Opcode::new(0x7203); // Add 03 to reg 2 (0).
    c8.decode_execute();

    assert_eq!(c8.registers[1], 0xA2);
    assert_eq!(c8.registers[2], 0x03);

    // Subtract nothing and check that values are not updated.
    c8.opcode = Opcode::new(0x8135);
    c8.decode_execute();

    assert_eq!(c8.registers[1], 0xA2);
    assert_eq!(c8.registers[2], 0x03);

    // Subtract 2 (03) from 1 (A2), make sure underflow was not reported.
    c8.opcode = Opcode::new(0x8125);
    c8.decode_execute();

    assert_eq!(c8.registers[1], 0x9F);
    assert_eq!(c8.registers[2], 0x03);
    assert_eq!(c8.registers[0xF], 1, "Register underflow was falsely reported!");

    // Finally, subtract 1 (9F) from 2 (3), check for underflow.
    c8.opcode = Opcode::new(0x8215);
    c8.decode_execute();

    assert_eq!(c8.registers[1], 0x9F);
    assert_eq!(c8.registers[2], 0x64);
    assert_eq!(c8.registers[0xF], 0, "Register underflow was falsely reported!");
}

#[test]
fn shift() {
    let mut c8 = Chip8::new(true);

    run_opcode(&mut c8, 0x7101); // Load register 1 with 1.
    assert_eq!(c8.registers[1], 1);

    // Shift it left.
    run_opcode(&mut c8, 0x819E); // Here, 9 can be substituted with anything.
    assert_eq!(c8.registers[1], 2);
    assert_eq!(c8.registers[0xF], 0, "MSB of shifted number was not 0!");

    // Now, shift it right twice.
    run_opcode(&mut c8, 0x8176);
    run_opcode(&mut c8, 0x8166);
    assert_eq!(c8.registers[1], 0);
    assert_eq!(c8.registers[0xF], 1, "MSB of shifted number was not 1!");
}
