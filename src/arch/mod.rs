mod emulator;
mod instruction_set;
mod opcode;
pub mod chip8;

pub use emulator::Emulator;
use instruction_set::InstructionSet;
use opcode::Opcode;