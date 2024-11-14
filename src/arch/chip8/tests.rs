use super::*;

fn run_opcode(c8: &mut Chip8, instruction: u16) {
    c8.opcode = Opcode::new(instruction);
    c8.decode_execute();
}

#[test]
fn setup() {
    let mut c8 = Chip8::new(true);
    assert_eq!(c8.pc, 0x200);

    let fontset_clear = c8.fontset.iter().all(|x| *x == 0);
    assert_eq!(fontset_clear, false);

    // Load a game and assert some well-known values were loaded into memory.
    c8.load_game(String::from("c8games/PONG2")).unwrap();
    assert_eq!(c8.memory[0x200], 0x22);
    assert_eq!(c8.memory[0x307], 0xEE);
}

#[test]
fn skip_instruction() {
    let mut c8 = Chip8::new(true);

    // First, add the literal (A3) to a register.
    run_opcode(&mut c8, 0x71A3);
    c8.increment_pc();

    // Now, check the PC.
    assert_eq!(c8.pc, 0x202);

    // Now, check that an instruction is skipped when
    // comparing the literal.
    run_opcode(&mut c8, 0x31A3);
    c8.increment_pc();
    assert_eq!(c8.pc, 0x206);

    // Now, check that an instruction is NOT skipped
    // when comparing the same literal.
    run_opcode(&mut c8, 0x41A3);
    c8.increment_pc();
    assert_eq!(c8.pc, 0x208);

    // Now, check that comparing two identical registers
    // leads to an instruction skip.
    run_opcode(&mut c8, 0x72A3); // Add literal to another register.
    c8.increment_pc();
    run_opcode(&mut c8, 0x5120);
    c8.increment_pc();
    assert_eq!(c8.pc, 0x20E);
}

#[test]
fn clear_screen() {
    let mut c8 = Chip8::new(true);

    // Draw something to the screen and assert that
    // some pixels were set.
    run_opcode(&mut c8, 0xD324);
    let pixels = c8.screen.get_pixels();

    // If all pixels are false, clear is true.
    let clear = pixels.iter().all(|x| x.iter().all(|&y| !y));
    assert_eq!(clear, false, "DrawSprite failed to draw the screen!");

    // Now, clear the screen, and check that it is empty.
    run_opcode(&mut c8, 0x00E0);
    let pixels = c8.screen.get_pixels();

    let clear = pixels.iter().all(|x| x.iter().all(|&y| !y));
    assert_eq!(clear, true, "ClearScreen failed to clear the screen!");
}

#[test]
fn call_return() {
    let mut c8 = Chip8::new(true);

    // Make sure the stack is initially empty.
    assert_eq!(c8.sp, 0);
    for val in c8.stack {
        assert_eq!(val, 0, "Found non-zero value in stack");
    }

    // Call a program at 789 and check the stack.
    run_opcode(&mut c8, 0x2789);
    assert_eq!(c8.sp, 1);
    assert_eq!(c8.stack[0], 0x200);
    for val in c8.stack[1..].iter() {
        assert_eq!(*val, 0, "Found non-zero value in upper part of stack");
    }

    // Return from that program and make sure
    // variables are reset.
    run_opcode(&mut c8, 0x00EE);
    assert_eq!(c8.sp, 0);
    for val in c8.stack[1..].iter() {
        assert_eq!(*val, 0, "Found non-zero value in stack");
    }
}

#[test]
fn add() {
    let mut c8 = Chip8::new(true);

    run_opcode(&mut c8, 0x7212);
    // Test that value is now correct.
    assert_eq!(c8.registers[2], 0x12);

    // Test the max value in a different register.
    run_opcode(&mut c8, 0x73FF);
    assert_eq!(c8.registers[3], 0xFF);

    // Now, add one, and test overflow.
    run_opcode(&mut c8, 0x7301);
    assert_eq!(c8.registers[3], 0x00);
}

#[test]
fn add_with_carry() {
    let mut c8 = Chip8::new(true);

    // Test adding the max value without overflow.
    run_opcode(&mut c8, 0x73FF); // Add FF to reg 3 (0).
    run_opcode(&mut c8, 0x8374); // Add reg 7 (0) to reg 3.

    // Test that values are now correct.
    assert_eq!(c8.registers[3], 0xFF);
    assert_eq!(c8.registers[0xF], 0);

    // Now, add one, and test overflow.
    run_opcode(&mut c8, 0x7401); // Add 1 to reg 4 (0).
    run_opcode(&mut c8, 0x8344); // Add reg 4 (1) to reg 3 (FF).

    assert_eq!(c8.registers[3], 0);
    // Test that the overflow register is correctly set.
    assert_eq!(c8.registers[0xF], 1);
}

#[test]
fn sub() {
    let mut c8 = Chip8::new(true);

    run_opcode(&mut c8, 0x71A2); // Add A2 to reg 1 (0).
    run_opcode(&mut c8, 0x7203); // Add 03 to reg 2 (0).
    assert_eq!(c8.registers[1], 0xA2);
    assert_eq!(c8.registers[2], 0x03);

    // Subtract nothing and check that values are not updated.
    run_opcode(&mut c8, 0x8135);
    assert_eq!(c8.registers[1], 0xA2);
    assert_eq!(c8.registers[2], 0x03);

    // Subtract 2 (03) from 1 (A2), make sure underflow was not reported.
    run_opcode(&mut c8, 0x8125);
    assert_eq!(c8.registers[1], 0x9F);
    assert_eq!(c8.registers[2], 0x03);
    assert_eq!(
        c8.registers[0xF], 1,
        "Register underflow was falsely reported!"
    );

    // Finally, subtract 1 (9F) from 2 (3), check for underflow.
    run_opcode(&mut c8, 0x8215);
    assert_eq!(c8.registers[1], 0x9F);
    assert_eq!(c8.registers[2], 0x64);
    assert_eq!(
        c8.registers[0xF], 0,
        "Register underflow was falsely reported!"
    );
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

#[test]
fn save_restore_registers() {
    let mut c8 = Chip8::new(true);

    run_opcode(&mut c8, 0x71A1); // Reg 1 has A1.
    run_opcode(&mut c8, 0x7206); // Reg 2 has 06.
    run_opcode(&mut c8, 0x76D4); // Reg 6 has D4.
    assert_eq!(c8.registers[1], 0xA1);
    assert_eq!(c8.registers[2], 0x06);
    assert_eq!(c8.registers[6], 0xD4);

    // Set the index register to our memory save location
    // (here, arbitrarily pick 0x345).
    run_opcode(&mut c8, 0xA345);
    assert_eq!(c8.index_reg, 0x345);

    // Load registers (up to register 6) into memory.
    run_opcode(&mut c8, 0xF655);
    assert_eq!(c8.memory[0x346], 0xA1);
    assert_eq!(c8.memory[0x347], 0x06);
    assert_eq!(c8.memory[0x34B], 0xD4);

    // Now, change registers 1 and 5.
    run_opcode(&mut c8, 0x7101);
    run_opcode(&mut c8, 0x75DD);
    assert_eq!(c8.registers[1], 0xA2);
    assert_eq!(c8.registers[5], 0xDD);
    assert_eq!(
        c8.index_reg, 0x345,
        "Index register was spuriously updated!"
    );

    // Now, reload our registers with memory contents and check them.
    run_opcode(&mut c8, 0xF665);
    assert_eq!(c8.registers[1], 0xA1);
    assert_eq!(c8.registers[2], 0x06);
    assert_eq!(c8.registers[5], 0x00);
    assert_eq!(c8.registers[6], 0xD4);
    assert_eq!(
        c8.index_reg, 0x345,
        "Index register was spuriously updated!"
    );
}
