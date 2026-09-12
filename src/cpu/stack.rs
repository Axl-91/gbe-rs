use crate::cpu::{
    Cpu,
    instruction::{StackInstruction, StackRegister},
};

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

    pub fn execute_stack(&mut self, instruction: StackInstruction) {
        match instruction {
            StackInstruction::Push(register) => {
                let sp = self.registers.get_sp();
                let new_sp = sp.wrapping_sub(2);

                let value = self.get_stack_register(&register);
                let lower_value = value as u8;
                let higher_value = (value >> 8) as u8;

                self.bus.write(sp.wrapping_sub(1), higher_value);
                self.bus.write(new_sp, lower_value);

                self.registers.set_sp(new_sp);
            }
            StackInstruction::Pop(register) => {
                let sp = self.registers.get_sp();
                let new_sp = sp.wrapping_add(2);

                let lower_value = self.bus.read(sp);
                let higher_value = self.bus.read(sp.wrapping_add(1));

                let value = u16::from_le_bytes([lower_value, higher_value]);

                self.set_stack_register(&register, value);
                self.registers.set_sp(new_sp);
            }
        }
    }
}
