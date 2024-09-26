#![allow(dead_code)]
#![allow(unused_variables)]
use crate::gfx::{Interactible, Screen};
use crate::arch::{Emulator, Opcode, InstructionSet};

#[cfg(test)]
mod tests;

pub struct Chip8 {
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
    screen: Screen,
    fontset: [u8; 80],
    draw_flag: bool,

    // Debug components.
    debug: bool,
    count: i32,
    cycle_rate: i32, // should be a time duration
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

    fn jump_with_offset(&mut self) {
        self.pc = self.opcode.literal + u16::from(self.registers[0]);
        self.update_pc_cycles = 0;
    }

    fn skip_if_eq_literal(&mut self) {
        // Here, the literal is just lower bits of value.
        let literal = (self.opcode.value & 0xFF) as u8;
        if self.registers[self.opcode.xreg] == literal {
            self.update_pc_cycles = 4; // Skip an instruction.
        }
    }

    fn skip_if_not_eq_literal(&mut self) {
        // Here, the literal is just lower bits of value.
        let literal = (self.opcode.value & 0xFF) as u8;
        if self.registers[self.opcode.xreg] != literal {
            self.update_pc_cycles = 4; // Skip an instruction.
        }
    }

    fn skip_if_eq_reg(&mut self) {
        if self.registers[self.opcode.xreg] == self.registers[self.opcode.yreg] {
            self.update_pc_cycles = 4;
        }
    }

    fn skip_if_not_eq_reg(&mut self) {
        if self.registers[self.opcode.xreg] != self.registers[self.opcode.yreg] {
            self.update_pc_cycles = 4;
        }
    }

    fn skip_if_key_pressed(&mut self) {

    }

    fn skip_if_key_not_pressed(&mut self) {

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

    fn save_binary_coded_decimal(&mut self) {
        let val = self.registers[self.opcode.xreg];

        // Store the decimal representation of val in memory so that
        // the hundreths digit of the value is in Mem[Index],
        // the tenths digit is in Mem[Index+1], and
        // the ones digit is in Mem[Index+2].
        self.memory[self.index_reg as usize] = val / 100;
        self.memory[(self.index_reg + 1) as usize] = (val / 10) % 10;
        self.memory[(self.index_reg + 2) as usize] = (val % 100) % 10;
    }

    // Manipulating special registers.
    fn add_reg_to_index_reg(&mut self) {
        self.index_reg += u16::from(self.registers[self.opcode.xreg]);
    }

    fn set_index_reg_to_literal(&mut self) {
        self.index_reg = self.opcode.literal;
    }

    fn get_delay_timer(&mut self) {
        self.registers[self.opcode.xreg] = self.delay_timer;
    }

    fn set_delay_timer(&mut self) {
        self.delay_timer = self.registers[self.opcode.xreg];
    }

    fn set_sound_timer(&mut self) {
        self.sound_timer = self.registers[self.opcode.xreg];
    }

    // Context switching.
    fn save_registers(&mut self) {
        // Store all registers up to last register in memory,
        // starting in memory at the location in the index register.
        for (loc, reg) in (self.index_reg..).zip(0..self.opcode.xreg) {
            // TODO: Technically overflow could happen here?
            self.memory[usize::from(loc)] = self.registers[reg];
        }
    }

    fn restore_registers(&mut self) {
        // Load all registers up to last register from memory,
        // starting in memory at the location in the index register.
        for (loc, reg) in (self.index_reg..).zip(0..self.opcode.xreg) {
            // TODO: overflow
            self.registers[reg] = self.memory[usize::from(loc)];
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
            screen: Screen::new(640, 480, 64, 32, String::from("Chip-8 Emulator")),
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

    fn decode_execute(&mut self) {
        self.update_pc_cycles = 2; // unless overridden
        let value = self.opcode.value;
        let lower_value = value & 0xFF;

        if self.debug {
            println!("Registers: {:?}", self.registers);
            println!("Executing opcode: {}", self.opcode);
        }

        match value >> 12 {
            0x0 => match lower_value {
                0xEE => self.r#return(),
                _ => self.unknown_instruction(),
            },
            0x1 => self.jump(),
            0x2 => self.call(),
            0x3 => self.skip_if_eq_literal(),
            0x4 => self.skip_if_not_eq_literal(),
            0x5 => self.skip_if_eq_reg(),
            0x6 => self.set_reg_to_literal(),
            0x7 => self.add(),
            0x8 => match value & 0xF { // *NOT* lower_value!
                0x0 => self.set_reg_to_reg(),
                0x1 => self.or(),
                0x2 => self.and(),
                0x3 => self.xor(),
                0x4 => self.add_with_carry(),
                0x5 => self.sub_y_from_x(),
                0x6 => self.shift_right(),
                0x7 => self.sub_x_from_y(),
                0xE => self.shift_left(),
                _ => self.unknown_instruction(),
            },
            0x9 => self.skip_if_not_eq_reg(),
            0xA => self.set_index_reg_to_literal(),
            0xB => self.jump_with_offset(),
            0xE => match lower_value {
                0x9E => self.skip_if_key_pressed(),
                0xA1 => self.skip_if_key_not_pressed(),
                _ => self.unknown_instruction(),
            },
            0xF => match lower_value {
                0x07 => self.get_delay_timer(),
                0x15 => self.set_delay_timer(),
                0x18 => self.set_sound_timer(),
                0x1E => self.add_reg_to_index_reg(),
                0x33 => self.save_binary_coded_decimal(),
                0x55 => self.save_registers(),
                0x65 => self.restore_registers(),
                _ => self.unknown_instruction(),
            }
            _ => self.unknown_instruction(),
        }
    }

    fn increment_pc(&mut self) {
        self.pc += self.update_pc_cycles;
    }

    fn unknown_instruction(&self) {
        panic!("Unimplemented opcode: {}", self.opcode);
    }

}


impl Emulator for Chip8 {
    // TODO: should return error instead
    fn load_game(&mut self, file_path: String) -> Result<(), std::io::Error> {

        return Ok(());
    }

    fn test_init(&mut self) {
        self.screen.init();

        while self.screen.set_keys() {}
    }

    fn run() {}
}