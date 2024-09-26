#![allow(dead_code)]
use crate::gfx;
use crate::arch::opcode::Opcode;

struct Chip8 {
    // Core structural components.
    opcode: Opcode, // reference?
    memory: [u8; 4096],
    registers: [u8; 16],
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
    cycle_rate: i32, // should be a time duration
}

trait InstructionSet {
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

    fn set_reg_to_literal(&mut self) {
        let literal = (self.opcode.value & 0xFF) as u8;
        self.registers[self.opcode.xreg] += literal;
    }

    fn set_reg_to_reg(&mut self) {
        let literal = self.registers[self.opcode.yreg];
        self.registers[self.opcode.xreg] = literal;
    }

    fn add(&mut self) {
        let literal = (self.opcode.value & 0xFF) as u8;
        self.registers[self.opcode.xreg] = self.registers[self.opcode.xreg].wrapping_add(literal);
    }

    fn add_with_carry(&mut self) {
        let (sum, overflowed) =
            self.registers[self.opcode.xreg].overflowing_add(self.registers[self.opcode.yreg]);

        self.registers[self.opcode.xreg] = sum;
        self.registers[0xF] = overflowed as u8;
    }

    fn or(&mut self) {
        let opcode = &self.opcode;
        self.registers[opcode.xreg] = self.registers[opcode.xreg] | self.registers[opcode.yreg];
    }

    fn and(&mut self) {
        let opcode = &self.opcode;
        self.registers[opcode.xreg] = self.registers[opcode.xreg] & self.registers[opcode.yreg];
    }

    fn xor(&mut self) {
        let opcode = &self.opcode;
        self.registers[opcode.xreg] = self.registers[opcode.xreg] ^ self.registers[opcode.yreg];
    }

    fn sub_x_from_y(&mut self) {
        let (diff, underflowed) =
            self.registers[self.opcode.yreg].overflowing_sub(self.registers[self.opcode.xreg]);

        self.registers[self.opcode.xreg] = diff;
        self.registers[0xF] = !underflowed as u8; // inverted, save 0 on underflow
    }

    fn sub_y_from_x(&mut self) {
        let (diff, underflowed) =
            self.registers[self.opcode.xreg].overflowing_sub(self.registers[self.opcode.yreg]);

        self.registers[self.opcode.xreg] = diff;
        self.registers[0xF] = !underflowed as u8; // inverted, save 0 on underflow
    }

    fn shift_right(&mut self) {
        let val = self.registers[self.opcode.xreg];

        // Set VF to least significant bit of Xreg before shifting.
        self.registers[0xF] = val & 0x1;
        self.registers[self.opcode.xreg] = val >> 1;
    }

    fn shift_left(&mut self) {
        let val = self.registers[self.opcode.xreg];

        // Set VF to most significant bit of Xreg before shifting.
        self.registers[0xF] = (val >> 7) & 0x1;
        self.registers[self.opcode.xreg] = val << 1;
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
}

impl Chip8 {
    pub fn new(debug: bool) -> Chip8 {
        let mut c8 = Chip8 {
            opcode: Opcode::new(0), // will be replaced
            memory: [0; 4096],
            registers: [0; 16], // this is an emulator, we use wrapping arithmetic
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
    fn load_game(file_path: String) {}

    fn decode_exeucte(&mut self) {
        self.update_pc_cycles = 2; // unless overridden
        let value = self.opcode.value;

        if self.debug {
            println!("Registers: {:?}", self.registers);
            println!("Executing opcode: {}", self.opcode);
        }

        match value >> 12 {
            0x0 => match value & 0xFF {
                0xEE => self.r#return(),
                _ => panic!("Unimplemented opcode: {}", self.opcode),
            },
            0x1 => self.jump(),
            0x2 => self.call(),
            0x6 => self.set_reg_to_literal(),
            0x7 => self.add(),
            0x8 => match value & 0xF {
                0x0 => self.set_reg_to_reg(),
                0x1 => self.or(),
                0x2 => self.and(),
                0x3 => self.xor(),
                0x4 => self.add_with_carry(),
                0x5 => self.sub_y_from_x(),
                0x6 => self.shift_right(),
                0x7 => self.sub_x_from_y(),
                0xE => self.shift_left(),
                _ => panic!("Unimplemented opcode: {}", self.opcode),
            },
            _ => panic!("Unimplemented opcode: {}", self.opcode),
        }
    }
}
