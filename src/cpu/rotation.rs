use crate::cpu::{Cpu, instruction::RotationInstruction};

impl Cpu {
    fn set_rotation_flags(&mut self, carry: bool) {
        self.registers.set_zero(false);
        self.registers.set_subtract(false);
        self.registers.set_half_carry(false);
        self.registers.set_carry(carry);
    }

    pub fn execute_rotation(&mut self, instruction: RotationInstruction) {
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
}
