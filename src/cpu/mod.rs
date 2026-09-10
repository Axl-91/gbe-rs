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

    fn decrement_8bit(&mut self, register: u8) -> u8 {
        let new_register = register.wrapping_sub(1);

        self.registers.set_half_carry(register & 0x0F == 0x00);

        self.registers.set_zero(new_register == 0x00);
        self.registers.set_subtract(true);

        new_register
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

    fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::Nop => {}
            Instruction::Inc(register) => {
                let register_value = self.get_register8(&register);
                let new_value = self.increment_8bit(register_value);

                self.set_register8(&register, new_value);
            }
            Instruction::Dec(register) => {
                let register_value = self.get_register8(&register);
                let new_value = self.decrement_8bit(register_value);

                self.set_register8(&register, new_value);
            }
            Instruction::Load8(register) => {
                let value = self.fetch();
                self.set_register8(&register, value);
            }
        }
    }

    pub fn step(&mut self) {
        let opcode = self.fetch();
        let instruction = decode(opcode);

        self.execute(instruction);
    }
}
