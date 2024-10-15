pub mod chip8;
mod emulator;
mod instruction_set;
mod opcode;

pub use emulator::Emulator;
use instruction_set::InstructionSet;
use opcode::Opcode;
