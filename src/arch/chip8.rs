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