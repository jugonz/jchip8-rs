struct Opcode {
    value:   u16,
    xreg:    u8, // "X" register in opcode table. 0 if unused, max 16.
    yreg:    u8, // 0 if unused.
    literal: u16, // Last three hex digits of our opcode.
}

impl Opcode {
    pub fn new(value: u16) -> Opcode {
        Opcode {
            value,
            xreg: ((value >> 8) & 0xF) as u8,
            yreg: ((value >> 4) & 0xF) as u8,
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

#[derive(Default)]
struct Chip8 {
    // Core structural components.
    opcode: Opcode, // reference?
    // memory: [u8; 4096],
    registers: [u8; 16],
    // index_reg: u16,
    pc: u16,
    delay_timer: u8,
    sound_timer: u8,
    // stack: [u16; 16],
    sp: u16,
    //rando: // PRNG
    update_pc: u16, // Amount of cycles to update PC.

    // Interactive components.

    // Debug components.
    debug: bool,
    count: i32,
    cycle_rate: i32 // should be a time duration

}

impl Chip8 {
    pub fn new(debug: bool) -> Chip8 {
        let c8 = Chip8 {
            opcode: Opcode::new(0), // will be replaced
            pc: 0x200, // Starting PC is static.
            // TODO initialize random ng
            debug: debug,
            cycle_rate: 1024, // TODO fix
            ..Default::default()
        };

        c8
    }
}