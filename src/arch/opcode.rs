use std::fmt::{Display, Formatter, Result};

// Don't allow initialization of an Opcode outside of this file.
#[non_exhaustive]
#[derive(Default, PartialEq)]
/// A struct that represents a single instruction for the emulator to run.
///
/// Members of `Opcode` are public for easier visibility,
/// but `Opcode` instances (other than the Default instance)
/// cannot be created by anything other than `Opcode::new()`.
pub struct Opcode {
    // The raw numerical value of the instruction.
    pub value: u16,
    // The 'X' (lower) register named in the instruction.
    // 0 if unused, max 16.
    // Registers are usize because rust forces indexing to be as usize,
    // even when the indexes are of a smaller type and thus are in
    // (compile-time) bounds.
    pub xreg: usize,
    // The 'Y' (higher) register named in the instruction.
    // 0 if unused, max 16.
    pub yreg: usize,
    // The last three hex digits of our instruction.
    // This is frequently a direct numerical value (a 'literal').
    pub literal: u16,
}

impl Opcode {
    /// Create a new `Opcode` from a numerical value.
    ///
    /// It is up to the caller to ensure this value represents
    /// a valid instruction for the emulator to execute.
    pub fn new(value: u16) -> Opcode {
        let xreg = ((value >> 8) & 0xF) as usize;
        let yreg = ((value >> 4) & 0xF) as usize;
        let literal = value & 0xFFF;

        Opcode {
            value,
            xreg,
            yreg,
            literal,
        }
    }
}

impl Display for Opcode {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "Value: 0x{:x} Xreg: {} Yreg: {} Literal: {}",
            self.value, self.xreg, self.yreg, self.literal
        )
    }
}
