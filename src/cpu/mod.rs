#![allow(dead_code)]

mod instruction;
mod registers;

#[cfg(test)]
mod tests;

use crate::{cpu::instruction::*, memory::MemoryBus};
use registers::Registers;

const GAME_ENTRY_POINT: u16 = 0x0100;

pub struct Cpu {
    registers: Registers,
    bus: MemoryBus,
}

impl Cpu {
    pub fn new(bus: MemoryBus) -> Self {
        let mut registers = Registers::new();
        registers.set_pc(GAME_ENTRY_POINT);

        Self { registers, bus }
    }

    fn fetch(&mut self) -> u8 {
        let pc = self.registers.get_pc();
        let opcode = self.bus.read(pc);

        self.registers.set_pc(pc + 1);
        opcode
    }

    fn increment_8bit(&mut self, register: u8) -> u8 {
        let new_register = register.wrapping_add(1);

        self.registers.set_half_carry(register & 0x0F == 0x0F);

        self.registers.set_zero(new_register == 0x00);
        self.registers.set_subtract(false);

        new_register
    }

    fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::Nop => {}
            Instruction::IncB => {
                let b = self.registers.get_b();
                let new_b = self.increment_8bit(b);
                self.registers.set_b(new_b);
            }
        }
    }

    pub fn step(&mut self) {
        let opcode = self.fetch();
        let instruction = decode(opcode);

        self.execute(instruction);
    }
}
