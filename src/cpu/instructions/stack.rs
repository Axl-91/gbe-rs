//! Game Boy CPU stack operations.
//!
//! Implements stack instructions and provides stack manipulation operations.

use crate::cpu::{Cpu, StackInstruction, StackRegister};

impl Cpu {
    fn get_stack_register(&self, register: &StackRegister) -> u16 {
        match register {
            StackRegister::AF => self.registers.get_af(),
            StackRegister::BC => self.registers.get_bc(),
            StackRegister::DE => self.registers.get_de(),
            StackRegister::HL => self.registers.get_hl(),
        }
    }

    fn set_stack_register(&mut self, register: &StackRegister, value: u16) {
        match register {
            // The lower 4 bits of register F are always zero
            StackRegister::AF => self.registers.set_af(value & 0xFFF0),
            StackRegister::BC => self.registers.set_bc(value),
            StackRegister::DE => self.registers.set_de(value),
            StackRegister::HL => self.registers.set_hl(value),
        }
    }

    pub(crate) fn push_into_sp(&mut self, value: u16) {
        let sp = self.registers.get_sp();
        let new_sp = sp.wrapping_sub(2);

        let lower_value = value as u8;
        let higher_value = (value >> 8) as u8;

        let high_address = sp.wrapping_sub(1);
        self.tick_write(high_address, higher_value);
        self.handle_oam_write_corruption(high_address);

        self.tick_write(new_sp, lower_value);
        self.handle_oam_write_corruption(new_sp);

        self.registers.set_sp(new_sp);
    }

    pub(crate) fn pop_from_sp(&mut self) -> u16 {
        let sp = self.registers.get_sp();
        let new_sp = sp.wrapping_add(2);

        let lower_value = self.tick_read(sp);
        self.handle_oam_read_write_corruption(sp);

        let high_address = sp.wrapping_add(1);
        let higher_value = self.tick_read(high_address);
        self.handle_oam_read_write_corruption(high_address);

        self.registers.set_sp(new_sp);

        u16::from_le_bytes([lower_value, higher_value])
    }

    /// Executes a CPU stack instruction.
    pub(crate) fn execute_stack(&mut self, instruction: StackInstruction) -> u8 {
        let t_cycles = instruction.t_cycles();
        match instruction {
            StackInstruction::Push(register) => {
                let value = self.get_stack_register(&register);
                let sp = self.registers.get_sp();

                self.tick_internal(1);
                self.handle_oam_write_corruption(sp);

                self.push_into_sp(value);
            }
            StackInstruction::Pop(register) => {
                let value = self.pop_from_sp();
                self.set_stack_register(&register, value);
            }
        }
        t_cycles
    }
}
