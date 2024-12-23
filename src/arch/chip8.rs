use super::{Emulator, InstructionSet, Opcode};
#[cfg(not(test))]
use crate::gfx::Hardware;
#[cfg(test)]
use crate::gfx::MockHardware;
use crate::gfx::{Drawable, Interactible, Screen, SetKeysResult};

use std::io::{Error, ErrorKind, Write};
use std::{fmt, fs, thread, time};

use serde::{Deserialize, Serialize};
use serde_json::error::Category;
use serde_with::serde_as;

// Emulator constants.
const NO_GAME_LOADED: &str = "No game loaded";
const DEFAULT_TITLE: &str = "Chip-8 Emulator";
const TITLE_PREFIX: &str = "chip8";
const START_PC: u16 = 0x200;
const CYCLE_RATE: u64 = 1666667; // ~60hz

#[cfg(test)]
mod tests;

// A simple abstraction of our Hardware types
// to avoid calling SDL methods during testing
// (see `MockHardware` for more info).
#[cfg(test)]
type Hw = MockHardware;
#[cfg(not(test))]
type Hw = Hardware;

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct Chip8 {
    // Core structural components.
    #[serde(skip)]
    opcode: Opcode,
    #[serde_as(as = "[_; 4096]")]
    // Core memory.
    // [0x0, START_PC) are reserved for our own use.
    memory: [u8; 4096],
    registers: [u8; 16],
    index_reg: u16,
    pc: u16,
    // A timer for emulated programs to use,
    // decremented once per cycle.
    delay_timer: u8,
    // A sound timer for emulated programs to use,
    // also decremented once per cycle.
    // We are responsible for emitting a sound when it hits zero.
    sound_timer: u8,
    stack: [u16; 16],
    sp: u8,
    // The amount of cycles to update the PC at the end of this cycle.
    update_pc_cycles: u16,
    // How fast to run one cycle in nanoseconds.
    cycle_rate: u64,

    // Interactive components.
    screen: Screen,
    #[serde(skip)]
    // The Interactible portion of the emulator.
    hardware: Hw,
    #[serde_as(as = "[_; 80]")]
    // Essentially hardcoded fonts to draw with.
    // We could skip serializing this, but it would require a
    // Default implementation for a fixed-size array of 80 u8's,
    // so we just allow it to be serialized.
    fontset: [u8; 80],
    draw_flag: bool,

    #[serde(skip)]
    // The game title (for use in the displayed window's title).
    game_title: String,

    #[serde(skip)]
    // Path to save a game state to (or overwrite), if any.
    save_state_path: Option<String>,

    // Debug components.
    #[serde(skip)]
    debug: bool,
    count: u64,
}

// The implementation of hardware instructions for the Chip8 platform.
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
                // (hedging against illegal code in the emulated program)
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
        self.update_pc_cycles = 0; // Since we just changed PC manually.
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
        self.registers[0xF] = !underflowed as u8; // Inverted, save 0 on underflow.
    }

    fn sub_y_from_x(&mut self) {
        let (diff, underflowed) =
            self.registers[self.opcode.xreg].overflowing_sub(self.registers[self.opcode.yreg]);

        self.registers[self.opcode.xreg] = diff;
        self.registers[0xF] = !underflowed as u8; // Inverted, save 0 on underflow.
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
        let mask = self.opcode.value as u8; // "as u8" chops to 0xFF for us.
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
                panic!("Cannot save register {reg} to memory location {loc}: out of bounds!");
            }

            self.memory[loc] = self.registers[reg];
        }
    }

    fn restore_registers(&mut self) {
        // Load all registers up to AND INCLUDING the last register from memory,
        // starting in memory at the location in the index register.
        for (loc, reg) in (usize::from(self.index_reg)..).zip(0..=self.opcode.xreg) {
            if loc >= self.memory.len() {
                panic!("Cannot load register {reg} from memory location {loc}: out of bounds!");
            }

            self.registers[reg] = self.memory[loc];
        }
    }

    // Save state handling.
    fn save_state(&mut self) {
        if let Some(path) = self.save_state_path.clone() {
            if let Err(error) = self.to_state(&path) {
                if self.debug {
                    println!("Failed to save state: {error}");
                }
            }
        }
    }
}

// Mostly useful for debugging.
impl fmt::Display for Chip8 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Opcode: {}, Memory: {:?}, Registers: {:?} Index Reg: {} \
            PC: {} Delay: {} Sound: {}, Stack: {:?}, SP: {}, \
            UPC: {}, Screen: ({}), DF: {}, Save Path: {:?}, Count: {}",
            self.opcode,
            self.memory,
            self.registers,
            self.index_reg,
            self.pc,
            self.delay_timer,
            self.sound_timer,
            self.stack,
            self.sp,
            self.update_pc_cycles,
            self.screen,
            self.draw_flag,
            self.save_state_path,
            self.count
        )
    }
}

// The default values for Chip8's members.
// We choose to define them here instead of inside an initialization function
// so that serde / serde_json can populate them as well when reading
// a state from disk (which does not store all of these members).
//
// Note that the default Hw instance / debug / opcode / save_state_path members
// are placeholders and must be overridden when using this default.
impl Default for Chip8 {
    fn default() -> Chip8 {
        let screen = Screen::default();
        let hardware = Hw::new(&screen, false, NO_GAME_LOADED);
        let mut c8 = Chip8 {
            opcode: Opcode::default(), // Will be replaced at fetch_opcode() time.

            memory: [0; 4096],
            registers: [0; 16], // We use wrapping arithmetic.
            index_reg: 0,
            pc: START_PC,
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
            cycle_rate: CYCLE_RATE,

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
    fn set_debug(&mut self, debug: bool) {
        // Override the debug value with a new one (useful when loading a state).
        self.hardware.debug = debug;
        self.debug = debug;
    }

    fn load_game(&mut self, file_path: &str) -> Result<(), Error> {
        // Load a game file from disk (without a saved state,
        // but with an already-initialized Chip8 instance).

        // Set the game's title.
        self.hardware
            .set_title(&format!("{}: {}", TITLE_PREFIX, file_path))?; // Handles title errors.
        self.game_title = String::from(file_path);

        // Load the game into memory.
        let contents: Vec<u8> = fs::read(file_path)?; // Handles all read errors.
        for (index, value) in contents.iter().enumerate() {
            self.memory[usize::from(START_PC) + index] = *value; // Essentially memcpy().
        }

        Ok(())
    }

    fn from_state(
        file_path: &str,
        debug: bool,
        save_state_path: Option<String>,
    ) -> Result<Chip8, Error> {
        // Load a game's state from disk (this includes the game data itself).
        // Here we do not have an existing Chip8 instance and must create one with serde and friends.

        // Read the game state and deserialize it into a Chip8 instance.
        let contents: Vec<u8> = fs::read(file_path)?; // Return errors inline.
        let parsed_c8: Result<Chip8, serde_json::Error> = serde_json::from_slice(&contents);
        match parsed_c8 {
            Ok(mut c8) => {
                // Update state not settable from default().
                c8.hardware
                    .set_title(&format!("{}: {}", TITLE_PREFIX, file_path))?; // Handles title errors.

                // Update state overridden by the user.
                c8.save_state_path = save_state_path;
                c8.set_debug(debug); // Debug is not stored in the state, so this only enables it.

                // Draw the screen once to start.
                c8.hardware.update_display(&c8.screen);
                Ok(c8)
            }
            Err(error) => {
                // Serde was not able to deserialize the state into a valid Chip8 instance.
                match error.classify() {
                    // We allow I/O errors to pass through because they may indicate a problem
                    // on the host system (i.e. the path to the saved state is not present).
                    Category::Io => Err(Error::other(error)),
                    // We assume all Syntax/Data/Eof errors are due to malformed input.
                    _ => Err(Error::new(
                        ErrorKind::InvalidInput,
                        "Load state path does not appear to point to a valid saved state!",
                    )),
                }
            }
        }
    }

    fn to_state(&mut self, to_file_path: &str) -> Result<(), Error> {
        // Save a Chip8 instance to disk (where it can be loaded again later).

        let mut save_file = fs::File::create(to_file_path)?;
        match serde_json::to_vec(self) {
            Ok(serialized_c8) => save_file.write_all(&serialized_c8),
            Err(error) => Err(Error::other(error)),
        }
    }

    pub fn new(
        debug: bool,
        game_path: Option<String>,
        load_state_path: Option<String>,
        save_state_path: Option<String>,
    ) -> Result<Chip8, Error> {
        // Create a Chip8 instance, given one of {path to game, save state to load}.
        // Optionally, provide a path to save game states to (which may be the same
        // as the path to the save state to load, in case the user wants to overwrite it).

        if let Some(game) = game_path {
            // Start a game from scratch.
            // (A provided path to a game file *always* overrides a load-state.)
            let hardware = Hw::new(&Screen::default(), debug, DEFAULT_TITLE);
            let mut c8 = Chip8 {
                hardware,
                debug,
                save_state_path,
                ..Default::default()
            };

            c8.load_game(&game)?;
            Ok(c8)
        } else if let Some(state) = load_state_path {
            // Load an existing game's state.
            Self::from_state(&state, debug, save_state_path)
        } else {
            Err(Error::new(
                ErrorKind::NotFound,
                "Neither a game nor a load state path was specified. Please check usage with '-h'.",
            ))
        }
    }

    #[cfg(test)]
    pub fn tester(debug: bool) -> Chip8 {
        // Create a Chip8 instance for unit testing.
        // Why not use Hw::default() here? Really only to pass debug.
        let hardware = Hw::new(&Screen::default(), debug, DEFAULT_TITLE);
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
        // Decode and execute the current Opcode value.

        self.update_pc_cycles = 2; // Unless overridden.
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
        // Draw the screen, if required.
        if self.draw_flag {
            self.hardware.update_display(&self.screen);
            self.draw_flag = false;
        }
    }

    fn update_timers(&mut self) {
        // Update delay and sound timers,
        // and beep if the sound timer has reached zero.

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
        // Increment the PC the correct amount of cycles.
        self.pc += self.update_pc_cycles;
    }

    fn emulate_cycle(&mut self) -> bool {
        // Emulate one cycle of our operation.
        // Returns false if we decided to stop.

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

        // Continue to the next cycle.
        true
    }

    fn unknown_instruction(&self) {
        panic!("Unimplemented opcode: {}", self.opcode);
    }
}

impl Emulator for Chip8 {
    fn run(&mut self) {
        // Run the emulated device, returning only when the game or user quits.

        self.fetch_opcode();
        if self.opcode == Opcode::default() {
            // No game is loaded, so just exit.
            // (This is mostly useful when a 'game' has been loaded that does not
            // contain valid Chip8 instructions.)
            return;
        }
        self.hardware.init();

        while self.emulate_cycle() {
            // Emulate a cycle, and then wait the proper amount to match the cycle rate.
            thread::sleep(time::Duration::from_nanos(self.cycle_rate));
        }
    }
}
