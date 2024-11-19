use crate::arch::{Emulator, InstructionSet, Opcode};
use crate::gfx::{Drawable, Hardware, Interactible, Screen, SetKeysResult};
use serde::{Serialize, Deserialize};
use serde_with::serde_as;
use std::io::Write;

use std::{fs, thread, time};

const NO_GAME_LOADED: &str = "No game loaded";
const DEFAULT_TITLE: &str = "Chip-8 Emulator";
const TITLE_PREFIX: &str = "chip8";

#[cfg(test)]
mod tests;

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct Chip8 {
    // Core structural components.
    #[serde(skip)]
    opcode: Opcode,
    #[serde_as(as = "[_; 4096]")]
    memory: [u8; 4096], // [0x0, 0x200) are reserved for our own use.
    registers: [u8; 16],
    index_reg: u16,
    pc: u16,
    delay_timer: u8,
    sound_timer: u8,
    stack: [u16; 16],
    sp: u8,
    update_pc_cycles: u16, // Amount of cycles to update PC.
    cycle_rate: u64,       // How fast to run one cycle in nanoseconds.

    // Interactive components.
    screen: Screen,
    #[serde(skip)]
    hardware: Hardware, // Interactible and Drawable.
    #[serde_as(as = "[_; 80]")] // We could skip serializing this but there is no default.
    fontset: [u8; 80],  // Essentially hardcoded fonts to draw with.
    draw_flag: bool,

    #[serde(skip)]
    game_title: String,

    #[serde(skip)]
    save_state_path: Option<String>,

    // Debug components.
    #[serde(skip)]
    debug: bool,
    count: u64,
}

impl InstructionSet for Chip8 {
    fn clear_screen(&mut self) {
        self.screen.clear_all_pixels();
        self.draw_flag = true;
    }

    fn draw_sprite(&mut self) {
        let x_coord: u16 = self.registers[self.opcode.xreg].into();
        let y_coord: u16 = self.registers[self.opcode.yreg].into();
        let height: u16 = self.opcode.value & 0xF;
        let width: u16 = 8; // Width is hardcoded on this platform.
        let shift_constant: u16 = 0x80; // Shifting 128 bits right allow us to check individual bits.

        self.registers[0xF] = 0; // Assume we don't unset any pixels.

        for y_line in 0..height {
            let pixel_offset: usize = (self.index_reg + y_line).into();
            let pixel: u16 = self.memory[pixel_offset].into();

            for x_line in 0..width {
                let x = x_coord + x_line;
                let y = y_coord + y_line;

                // If we need to draw this pixel...
                if (pixel & (shift_constant >> x_line)) > 0
                    && self.screen.in_bounds(u32::from(x), u32::from(y))
                {
                    // XOR the pixel, saving whether we set it here.
                    if self.screen.get_pixel(x, y) {
                        self.registers[0xF] = 1;
                    }
                    self.screen.xor_pixel(x, y);
                }
            }
        }

        self.draw_flag = true;
    }

    fn set_index_reg_to_sprite(&mut self) {
        let character = u16::from(self.registers[self.opcode.xreg]);
        // Number of sprites per character. (If this overflows, something is very very wrong...)
        let offset = (self.fontset.len() / self.hardware.get_keys().len()) as u16;

        // Set the index register to the location of the
        // first fontset sprite of the matching character.
        self.index_reg = character * offset;
    }

    fn call(&mut self) {
        self.stack[self.sp as usize] = self.pc;
        self.sp += 1; // Allow overflow to panic - the stack is only 16 entries anyway.

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
        let literal = self.opcode.value as u8;
        if self.registers[self.opcode.xreg] == literal {
            self.update_pc_cycles = 4; // Skip an instruction.
        }
    }

    fn skip_if_not_eq_literal(&mut self) {
        // Here, the literal is just lower bits of value.
        let literal = self.opcode.value as u8;
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
        if self
            .hardware
            .key_is_pressed(self.registers[self.opcode.xreg])
        {
            self.update_pc_cycles = 4;
        }
    }

    fn skip_if_key_not_pressed(&mut self) {
        if !self
            .hardware
            .key_is_pressed(self.registers[self.opcode.xreg])
        {
            self.update_pc_cycles = 4;
        }
    }

    fn set_reg_to_literal(&mut self) {
        let literal = self.opcode.value as u8; // Overflow is possible, but we ignore it.
        self.registers[self.opcode.xreg] = literal;
    }

    fn set_reg_to_reg(&mut self) {
        let literal = self.registers[self.opcode.yreg];
        self.registers[self.opcode.xreg] = literal;
    }

    fn add(&mut self) {
        let literal = self.opcode.value as u8; // Overflow is possible, but we ignore it.
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

    fn set_reg_random_mask(&mut self) {
        let mask = self.opcode.value as u8; // "as u8" chops to 0xFF for us
        let random_number = rand::random::<u8>();

        self.registers[self.opcode.xreg] = mask & random_number;
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

    fn get_key_press(&mut self) {
        let keyboard = self.hardware.get_keys();

        for (key, pressed) in keyboard.iter().enumerate() {
            if *pressed {
                // If key as u8 overflows u8, the instruction was invalid!
                self.registers[self.opcode.xreg] = key as u8;
                return;
            }
        }

        // Else, don't increment the PC, we'll wait another cycle for the key.
        self.update_pc_cycles = 0;
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
        // Store all registers up to AND INCLUDING the last register in memory,
        // starting in memory at the location in the index register.
        for (loc, reg) in (usize::from(self.index_reg)..).zip(0..=self.opcode.xreg) {
            if loc >= self.memory.len() {
                panic!("Cannot save save register {reg} to memory location {loc}: out of bounds!");
            }

            self.memory[loc] = self.registers[reg];
        }
    }

    fn restore_registers(&mut self) {
        // Load all registers up to AND INCLUDING the last register from memory,
        // starting in memory at the location in the index register.
        for (loc, reg) in (usize::from(self.index_reg)..).zip(0..=self.opcode.xreg) {
            if loc >= self.memory.len() {
                panic!("Cannot save load register {reg} from memory location {loc}: out of bounds!");
            }

            self.registers[reg] = self.memory[loc];
        }
    }

    // Save state handling.
    fn save_state(&mut self) {
        if let Some(path) = self.save_state_path.clone() {
            if let Err(error) = self.to_state(&path) {
                if self.debug {
                    println!("Saving state failed: {error}");
                }
            }
        }
    }
}

// impl Serialize for Chip8 {

//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer
//     {
//         let mut state = serializer.serialize_struct("Chip8", 4)?;
//         // MEMORY
//         state.serialize_seq("mm", &self.memory)?;
//         // REGISTERS
//         state.serialize_field("ir", &self.index_reg)?;
//         state.serialize_field("pc", &self.pc)?;
//         state.serialize_field("dt", &self.delay_timer)?;
//         state.serialize_field("st", &self.sound_timer)?;
//         // STACK
//         state.serialize_field("sp", &self.sp)?;
//         state.serialize_field("df", &self.draw_flag)?;

//         state.end()
//     }
// }

impl std::fmt::Display for Chip8 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f,
            "Opcode: {}, Memory: {:?}, Registers: {:?} Index Reg:{} \
            PC: {} Delay: {} Sound: {}, Stack: {:?}, SP: {}, \
            UPC: {}, DF: {}, Save Path: {:?}, Count: {}",
            self.opcode, self.memory, self.registers, self.index_reg,
            self.pc, self.delay_timer, self.sound_timer, self.stack, self.sp,
            self.update_pc_cycles, self.draw_flag, self.save_state_path, self.count)
    }
}

impl Default for Chip8 {
    fn default() -> Chip8 {
        let screen = Screen::new(640, 480, 64, 32);
        let hardware = Hardware::new(&screen, false, String::from(NO_GAME_LOADED));
        let mut c8 = Chip8 {
            opcode: Opcode::default(), // will be replaced

            memory: [0; 4096],
            registers: [0; 16], // this is an emulator, we use wrapping arithmetic
            index_reg: 0,
            pc: 0x200, // Starting PC is static.
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            sp: 0,
            update_pc_cycles: 0,

            screen,
            hardware,
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
            cycle_rate: 1666667, // 60hz

            game_title: String::from(NO_GAME_LOADED),
            save_state_path: None,

            debug: false,
            count: 0,
        };

        // Load the fontset into memory.
        for (item, value) in c8.fontset.iter().enumerate() {
            c8.memory[item] = *value;
        }

        c8
    }
}

impl Chip8 {
    fn load_game(&mut self, file_path: String) -> Result<(), std::io::Error> {
        self.hardware.set_title(format!("{}: {}", TITLE_PREFIX, file_path.clone()))?; // Handles title errors.
        self.game_title = file_path.clone();

        let contents: Vec<u8> = fs::read(file_path)?; // Consume file_path and handle all read errors.
        for (index, value) in contents.iter().enumerate() {
            self.memory[0x200 + index] = *value;
        }

        Ok(())
    }

    fn from_state(file_path: &str, save_state_path: Option<String>) -> Result<Chip8, std::io::Error> {
        let contents: Vec<u8> = fs::read(file_path)?; // Return errors inline.
        let parsed_c8: Result<Chip8, serde_json::Error> = serde_json::from_slice(&contents);
        match parsed_c8 {
            Ok(mut c8) => {
                c8.save_state_path = save_state_path;
                println!("Loaded state is: {c8}");
                c8.hardware.update_display(&c8.screen);
                Ok(c8)
            }
            Err(error) => Err(std::io::Error::other(error)),
        }
    }

    fn to_state(&mut self, to_file_path: &str) -> Result<(), std::io::Error> {
        let mut save_file = fs::File::create(to_file_path)?;
        match serde_json::to_vec(self) {
            Ok(serialized_c8) => {
                println!("Serialized state is: {serialized_c8:?}");
                println!("In-memory state is: {self}");
                let res = save_file.write_all(&serialized_c8);
                std::thread::sleep(std::time::Duration::from_nanos(1000000000));
                return res;
            }
            Err(error) => { Err(std::io::Error::other(error)) },
        }
    }


    pub fn new(debug: bool, game_path: Option<String>,
        load_state_path: Option<String>, save_state_path: Option<String>) -> Result<Chip8, std::io::Error> {
        if let Some(game) = game_path {
            let screen = Screen::new(640, 480, 64, 32);
            let hardware = Hardware::new(&screen, debug, String::from(DEFAULT_TITLE));
            let mut c8 = Chip8 {
                hardware,
                debug,
                save_state_path,
                ..Default::default()
            };

            c8.load_game(game)?;
            Ok(c8)
        } else if let Some(state) = load_state_path {
            Self::from_state(&state, save_state_path)
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "No path specified"))
        }

    }

    #[cfg(test)]
    pub fn tester(debug: bool) -> Chip8 {
        let screen = Screen::new(640, 480, 64, 32);
        let hardware = Hardware::new(&screen, debug, String::from(DEFAULT_TITLE));
        Chip8 {
            hardware,
            debug,
            ..Default::default()
        }
    }

    fn fetch_opcode(&mut self) {
        // Read the 8 bytes at Memory[PC], save them into a 16-bit variable
        // and shift them to the lower 8 bits.
        let mut new_opcode = (u16::from(self.memory[self.pc as usize])) << 8;
        // Then, read the 8 bytes at Memory[PC + 1],
        // and save them into the higher 8 bits of our variable.
        // Together, these bits are our complete opcode.
        new_opcode |= u16::from(self.memory[(self.pc + 1) as usize]);

        self.opcode = Opcode::new(new_opcode);
    }

    fn decode_execute(&mut self) {
        self.update_pc_cycles = 2; // unless overridden
        let value = self.opcode.value;
        let lower_value = value as u8;

        if self.debug {
            println!("Registers: {:?}", self.registers);
            println!("Executing opcode: {}", self.opcode);
        }

        // Make sure that xreg / yreg are sane,
        // since we don't have Opcode::new() check that for us
        // for easier indexing later on.
        let max_register = self.registers.len() - 1;
        if self.opcode.xreg > max_register || self.opcode.yreg > max_register {
            panic!("Invalid register in opcode: {}", self.opcode);
        }

        match value >> 12 {
            0x0 => match lower_value {
                0xE0 => self.clear_screen(),
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
            0x8 => match value & 0xF {
                // *NOT* lower_value!
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
            0xC => self.set_reg_random_mask(),
            0xD => self.draw_sprite(),
            0xE => match lower_value {
                0x9E => self.skip_if_key_pressed(),
                0xA1 => self.skip_if_key_not_pressed(),
                _ => self.unknown_instruction(),
            },
            0xF => match lower_value {
                0x07 => self.get_delay_timer(),
                0x0A => self.get_key_press(),
                0x15 => self.set_delay_timer(),
                0x18 => self.set_sound_timer(),
                0x1E => self.add_reg_to_index_reg(),
                0x29 => self.set_index_reg_to_sprite(),
                0x33 => self.save_binary_coded_decimal(),
                0x55 => self.save_registers(),
                0x65 => self.restore_registers(),
                _ => self.unknown_instruction(),
            },
            _ => self.unknown_instruction(),
        }
    }

    fn draw_screen(&mut self) {
        if self.draw_flag {
            self.hardware.update_display(&self.screen);
            self.draw_flag = false;
        }
    }

    fn update_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            print!("\x07"); // BEEP!
            let _ = std::io::stdout().flush(); // If this fails, it's not a catastrophe.
            self.sound_timer -= 1;
        }
    }

    fn increment_pc(&mut self) {
        self.pc += self.update_pc_cycles;
    }

    fn emulate_cycle(&mut self) -> bool {
        // Returns false if we decided to stop.
        // Emulate one cycle of our operation.
        self.fetch_opcode();
        if self.debug {
            println!("On cycle {}, at memory location {}", self.count, self.pc);
            self.count += 1;
        }

        self.decode_execute();
        self.draw_screen();
        match self.hardware.set_keys(&self.screen) {
            SetKeysResult::ShouldSaveState => self.save_state(),
            SetKeysResult::ShouldExit => return false,
            _ => (),
        }
        self.update_timers();
        self.increment_pc();

        return true;
    }

    fn unknown_instruction(&self) {
        panic!("Unimplemented opcode: {}", self.opcode);
    }
}

impl Emulator for Chip8 {
    fn run(&mut self) {
        self.fetch_opcode();
        if self.opcode == Opcode::default() {
            // No game is loaded, so just exit.
            return;
        }

        self.hardware.init();

        while self.emulate_cycle() {
            thread::sleep(time::Duration::from_nanos(self.cycle_rate));
        }
    }
}
