//! Game Boy CPU implementation.
//!
//! Handles instruction fetching, decoding, execution, and CPU registers.

mod arithmetic;
mod cb;
mod control;
mod instruction;
mod load;
mod registers;
mod rotation;
mod stack;
mod t_cycles;

#[cfg(test)]
mod tests;

use crate::{cpu::instruction::*, memory::MemoryBus};
use registers::Registers;

/// Address where the Game Boy starts executing the game after the boot sequence.
const GAME_ENTRY_POINT: u16 = 0x0100;

/// Represents the Game Boy CPU and its connection to the memory bus.
pub struct Cpu {
    registers: Registers,
    bus: MemoryBus,
}

impl Cpu {
    /// Creates a new CPU with initialized registers and the program counter
    /// set to the game's entry point.
    pub fn new(bus: MemoryBus) -> Self {
        let mut registers = Registers::new();
        registers.set_pc(GAME_ENTRY_POINT);

        Self { registers, bus }
    }

    /// Fetches the next opcode from memory and advances the program counter.
    fn fetch(&mut self) -> u8 {
        let pc = self.registers.get_pc();
        let opcode = self.bus.read(pc);

        self.registers.set_pc(pc + 1);
        opcode
    }

    fn get_register8(&self, register: &Register8) -> u8 {
        match register {
            Register8::A => self.registers.get_a(),
            Register8::B => self.registers.get_b(),
            Register8::C => self.registers.get_c(),
            Register8::D => self.registers.get_d(),
            Register8::E => self.registers.get_e(),
            Register8::H => self.registers.get_h(),
            Register8::L => self.registers.get_l(),
        }
    }

    fn set_register8(&mut self, register: &Register8, value: u8) {
        match register {
            Register8::A => self.registers.set_a(value),
            Register8::B => self.registers.set_b(value),
            Register8::C => self.registers.set_c(value),
            Register8::D => self.registers.set_d(value),
            Register8::E => self.registers.set_e(value),
            Register8::H => self.registers.set_h(value),
            Register8::L => self.registers.set_l(value),
        }
    }

    fn get_register16(&self, register: &Register16) -> u16 {
        match register {
            Register16::BC => self.registers.get_bc(),
            Register16::DE => self.registers.get_de(),
            Register16::HL => self.registers.get_hl(),
            Register16::SP => self.registers.get_sp(),
        }
    }

    fn set_register16(&mut self, register: &Register16, value: u16) {
        match register {
            Register16::BC => self.registers.set_bc(value),
            Register16::DE => self.registers.set_de(value),
            Register16::HL => self.registers.set_hl(value),
            Register16::SP => self.registers.set_sp(value),
        }
    }

    /// Sets the flags affected by an SP plus immediate offset operation.
    fn set_flags_sp_plus_immediate(&mut self, sp: u16, offset: i8) {
        let offset = offset as u8;

        self.registers.set_zero(false);
        self.registers.set_subtract(false);
        self.registers
            .set_half_carry((sp & 0x000F) + (offset as u16 & 0x000F) > 0x000F);

        self.registers
            .set_carry((sp & 0x00FF) + offset as u16 > 0x00FF);
    }

    /// Executes the instruction and returns its T-Cycles.
    fn execute(&mut self, instruction: Instruction) -> u8 {
        match instruction {
            Instruction::Nop => 4,

            Instruction::Load(instruction) => self.execute_load(instruction),

            Instruction::Arithmetic(instruction) => self.execute_arithmetic(instruction),

            Instruction::Stack(instruction) => self.execute_stack(instruction),

            Instruction::Rotation(instruction) => self.execute_rotation(instruction),

            Instruction::Control(instruction) => self.execute_control(instruction),
            Instruction::Cb => {
                let opcode = self.fetch();
                let instruction = decode_cb(opcode);

                self.execute_cb(instruction)
            }
        }
    }

    /// Fetches the opcode, decodes it and executes the instruction
    /// then returns its T-Cycles
    pub fn step(&mut self) -> u8 {
        let opcode = self.fetch();
        let instruction = decode(opcode);

        self.execute(instruction)
    }
}
