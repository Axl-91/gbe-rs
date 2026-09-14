//! Game Boy CPU rotation instructions.
//!
//! Implements accumulator rotation operations and their affected flags.

use crate::cpu::{
    Cpu,
    instruction::{CbRotation, RotationInstruction},
};

impl Cpu {
    fn set_rotation_flags(&mut self, carry: bool) {
        self.registers.set_zero(false);
        self.registers.set_subtract(false);
        self.registers.set_half_carry(false);
        self.registers.set_carry(carry);
    }

    /// Executes a CPU rotation instruction.
    pub(crate) fn execute_rotation(&mut self, instruction: RotationInstruction) {
        match instruction {
            RotationInstruction::Rlca => {
                let a = self.registers.get_a();

                let new_carry = a >> 7;
                let new_a = a.rotate_left(1);

                self.set_rotation_flags(new_carry != 0);
                self.registers.set_a(new_a);
            }
            RotationInstruction::Rla => {
                let a = self.registers.get_a();
                let carry = self.registers.get_carry() as u8;

                let new_carry = a >> 7;
                let new_a = (a << 1) | carry;

                self.set_rotation_flags(new_carry != 0);
                self.registers.set_a(new_a);
            }
            RotationInstruction::Rrca => {
                let a = self.registers.get_a();

                let new_carry = a & 0x01;
                let new_a = a.rotate_right(1);

                self.set_rotation_flags(new_carry != 0);
                self.registers.set_a(new_a);
            }
            RotationInstruction::Rra => {
                let a = self.registers.get_a();
                let carry = self.registers.get_carry() as u8;

                let new_carry = a & 0x01;
                let new_a = (a >> 1) | (carry << 7);

                self.set_rotation_flags(new_carry != 0);
                self.registers.set_a(new_a);
            }
        }
    }
    pub(crate) fn execute_cb_rotation(&mut self, instruction: CbRotation) {
        match instruction {
            CbRotation::Rlc(register) => {
                let value = self.get_register8(&register);

                let new_carry = value >> 7;
                let new_value = value.rotate_left(1);

                self.set_rotation_flags(new_carry != 0);
                self.set_register8(&register, new_value);
            }
            CbRotation::RlcFromHl => {
                let hl = self.registers.get_hl();
                let value = self.bus.read(hl);

                let new_carry = value >> 7;
                let new_value = value.rotate_left(1);

                self.set_rotation_flags(new_carry != 0);
                self.bus.write(hl, new_value);
            }
            CbRotation::Rrc(register) => {
                let value = self.get_register8(&register);

                let new_carry = value & 0x01;
                let new_value = value.rotate_right(1);

                self.set_rotation_flags(new_carry != 0);
                self.set_register8(&register, new_value);
            }
            CbRotation::RrcFromHl => {
                let hl = self.registers.get_hl();
                let value = self.bus.read(hl);

                let new_carry = value & 0x01;
                let new_value = value.rotate_right(1);

                self.set_rotation_flags(new_carry != 0);
                self.bus.write(hl, new_value);
            }
            CbRotation::Rl(register) => {
                let value = self.get_register8(&register);
                let carry = self.registers.get_carry() as u8;

                let new_carry = value >> 7;
                let new_value = (value << 1) | carry;

                self.set_rotation_flags(new_carry != 0);
                self.set_register8(&register, new_value);
            }
            CbRotation::RlFromHl => {
                let hl = self.registers.get_hl();
                let value = self.bus.read(hl);
                let carry = self.registers.get_carry() as u8;

                let new_carry = value >> 7;
                let new_value = (value << 1) | carry;

                self.set_rotation_flags(new_carry != 0);
                self.bus.write(hl, new_value);
            }
            CbRotation::Rr(register) => {
                let value = self.get_register8(&register);
                let carry = self.registers.get_carry() as u8;

                let new_carry = value & 0x01;
                let new_value = (value >> 1) | (carry << 7);

                self.set_rotation_flags(new_carry != 0);
                self.set_register8(&register, new_value);
            }
            CbRotation::RrFromHl => {
                let hl = self.registers.get_hl();
                let value = self.bus.read(hl);
                let carry = self.registers.get_carry() as u8;

                let new_carry = value & 0x01;
                let new_value = (value >> 1) | (carry << 7);

                self.set_rotation_flags(new_carry != 0);
                self.bus.write(hl, new_value);
            }
            CbRotation::Sla(_register) => {
                todo!();
            }
            CbRotation::SlaFromHl => {
                todo!();
            }
            CbRotation::Sra(_register) => {
                todo!();
            }
            CbRotation::SraFromHl => {
                todo!();
            }
            CbRotation::Swap(_register) => {
                todo!();
            }
            CbRotation::SwapFromHl => {
                todo!();
            }
            CbRotation::Srl(_register) => {
                todo!();
            }
            CbRotation::SrlFromHl => {
                todo!();
            }
        }
    }
}
