use crate::cpu::{Cpu, instruction::CbInstruction};

impl Cpu {
    /// Executes a CB-prefixed instruction.
    pub(crate) fn execute_cb(&mut self, instruction: CbInstruction) {
        match instruction {
            CbInstruction::Bit(bit_value, register) => {
                let test = (self.get_register8(&register) >> bit_value) & 0x01;

                self.registers.set_zero(test == 0);
                self.registers.set_subtract(false);
                self.registers.set_half_carry(true);
            }
            CbInstruction::BitFromHl(bit_value) => {
                let hl = self.registers.get_hl();
                let test = (self.bus.read(hl) >> bit_value) & 0x01;

                self.registers.set_zero(test == 0);
                self.registers.set_subtract(false);
                self.registers.set_half_carry(true);
            }
            CbInstruction::Res(bit_value, register) => {
                let zero_bit: u8 = !(0x01 << bit_value);
                let new_value = self.get_register8(&register) & zero_bit;

                self.set_register8(&register, new_value);
            }
            CbInstruction::ResFromHl(bit_value) => {
                let hl = self.registers.get_hl();
                let zero_bit: u8 = !(0x01 << bit_value);
                let new_value = self.bus.read(hl) & zero_bit;

                self.bus.write(hl, new_value);
            }
            CbInstruction::Set(bit_value, register) => {
                let one_bit: u8 = 0x01 << bit_value;
                let new_value = self.get_register8(&register) | one_bit;

                self.set_register8(&register, new_value);
            }
            CbInstruction::SetFromHl(bit_value) => {
                let hl = self.registers.get_hl();
                let one_bit: u8 = 0x01 << bit_value;
                let new_value = self.bus.read(hl) | one_bit;

                self.bus.write(hl, new_value);
            }
            CbInstruction::Rotation(instruction) => {
                self.execute_cb_rotation(instruction);
            }
        }
    }
}
