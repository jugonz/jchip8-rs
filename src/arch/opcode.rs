#[non_exhaustive] // don't allow init of Opcode outside this file
#[derive(PartialEq)]
pub struct Opcode {
    pub value: u16,
    // Registers are usize because rust forces indexing to be as usize,
    // even when the indexes are of a smaller type and thus are in (compile-time) bounds.
    pub xreg: usize,  // "X" register in opcode table. 0 if unused, max 16.
    pub yreg: usize,  // 0 if unused.
    pub literal: u16, // Last three hex digits of our opcode.
}

impl Opcode {
    pub fn new(value: u16) -> Opcode {
        let xreg = ((value >> 8) & 0xF) as usize;
        let yreg = ((value >> 4) & 0xF) as usize;

        Opcode {
            value,
            xreg,
            yreg,
            literal: value & 0xFFF,
        }
    }
}

impl std::fmt::Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Value: 0x{:x} Xreg: {} Yreg: {} Literal: {}",
            self.value, self.xreg, self.yreg, self.literal
        )
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
