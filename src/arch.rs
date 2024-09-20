use crate::gfx;

use std::num::Wrapping;

struct Opcode {
    value:   u16,
    // Registers are usize because rust forces indexing to be as usize,
    // even when the indexes are of a smaller type and thus are in (compile-time) bounds.
    // TODO: We will enforce that they are in bounds at execution time.
    xreg:    usize, // "X" register in opcode table. 0 if unused, max 16.
    yreg:    usize, // 0 if unused.
    literal: u16, // Last three hex digits of our opcode.
}

impl Opcode {
    pub fn new(value: u16) -> Opcode {
        let xreg = ((value >> 8) & 0xF) as usize;
        let yreg = ((value >> 4) & 0xF) as usize;

        Opcode {
            value,
            xreg,
            yreg,
            literal: value & 0xFFF
        }
    }
}

impl Default for Opcode {
    fn default() -> Opcode {
        Opcode {
            value: 0,
            xreg: 0,
            yreg: 0,
            literal: 0,
        }
    }
}

struct Chip8 {
    // Core structural components.
    opcode: Opcode, // reference?
    memory: [u8; 4096],
    registers: [Wrapping<u8>; 16],
    index_reg: u16,
    pc: u16,
    delay_timer: u8,
    sound_timer: u8,
    stack: [u16; 16],
    sp: u16,
    //rando: // PRNG
    update_pc_cycles: u16, // Amount of cycles to update PC.

    // Interactive components.
    screen: gfx::Screen,
    fontset: [u8; 80],
    draw_flag: bool,

    // Debug components.
    debug: bool,
    count: i32,
    cycle_rate: i32 // should be a time duration

}

trait InstructionSet {
    fn call(&mut self);
    fn r#return(&mut self);
    fn jump(&mut self);
    fn add(&mut self);
    fn or(&mut self);
}

impl InstructionSet for Chip8 {
    fn call(&mut self) {
        self.stack[self.sp as usize] = self.pc;
        self.sp += 1; // TODO: Overflow?

        self.pc = self.opcode.literal;
        self.update_pc_cycles = 0; // since we just changed PC manually
    }

    fn r#return(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp as usize];
    }

    fn jump(&mut self) {
        self.pc = self.opcode.literal;
        self.update_pc_cycles = 0;
    }

    fn add(&mut self) {
        let literal = (self.opcode.value & 0xFF) as u8;
        self.registers[self.opcode.xreg] += literal;
    }

    fn or(&mut self) {
        let opcode = &self.opcode;
        self.registers[opcode.xreg] =
            self.registers[opcode.xreg] | self.registers[opcode.yreg];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add() {
        let mut c8 = Chip8::new(false);

        c8.opcode = Opcode::new(0x7212);
        c8.decode_exeucte();

        // Test that value is now correct.
        assert_eq!(c8.registers[2].0, 0x12);

        // Test the max value in a different register.
        c8.opcode = Opcode::new(0x73FF);
        c8.decode_exeucte();

        // Test that value is now correct.
        assert_eq!(c8.registers[3].0, 0xFF);

        // Now, add one, and test overflow.
        c8.opcode = Opcode::new(0x7301);
        c8.decode_exeucte();

        // Test that value is now correct.
        assert_eq!(c8.registers[3].0, 0x00);
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

}

impl Chip8 {
    pub fn new(debug: bool) -> Chip8 {
        let mut c8 = Chip8 {
            opcode: Opcode::new(0), // will be replaced
            memory: [0; 4096],
            registers: [Wrapping(0u8); 16], // this is an emulator, we use wrapping arithmetic
            index_reg: 0,
            pc: 0x200, // Starting PC is static.
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            sp: 0,
            update_pc_cycles: 0,
            // TODO initialize random ng

            screen: gfx::Screen::new(640, 480, 64, 32, "Chip-8 Emulator".to_string()),
            fontset: [
                0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
                0x20, 0x60, 0x20, 0x20, 0x70, // 1
                0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
                0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
                0x90, 0x90, 0xF0, 0x10, 0x10, // 4
                0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
                0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
                0xF0, 0x10, 0x20, 0x40, 0x40, // 7
                0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
                0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
                0xF0, 0x90, 0xF0, 0x90, 0x90, // A
                0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
                0xF0, 0x80, 0x80, 0x80, 0xF0, // C
                0xE0, 0x90, 0x90, 0x90, 0xE0, // D
                0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
                0xF0, 0x80, 0xF0, 0x80, 0x80, // F
            ],
            draw_flag: false,

            debug,
            count: 0,
            cycle_rate: 1024, // TODO fix
        };

        // Load the fontset into memory.
        for item in c8.fontset.into_iter().enumerate() {
            let (i, v): (usize, u8) = item;
            c8.memory[i] = v;
        }

        c8
    }

    // TODO: should return error instead
    fn load_game(file_path: String) {


    }

    fn decode_exeucte(&mut self) {
        self.update_pc_cycles = 2; // unless overridden
        let value = self.opcode.value;

        match value >> 12 {
            0x0 => {
                match value & 0xFF {
                    0xEE => self.r#return(),
                    _ => (),
                }
            },
            0x1 => self.jump(),
            0x2 => self.call(),
            0x7 => self.add(),
            0x8 => {
                match value & 0xF {
                    0x1 => self.or(),
                    _ => (),
                }
            },
            _ => (),
        }
    }

}