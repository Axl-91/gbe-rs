use crate::cpu::{Cpu, instruction::ArithmeticInstruction};

impl Cpu {
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

    fn set_add_carry_flags8(&mut self, register: u8, value: u8) {
        self.registers
            .set_half_carry((register & 0x0F) + (value & 0x0F) > 0x0F);
        self.registers
            .set_carry((register as u16) + (value as u16) > 0xFF);
    }

    fn set_sub_borrow_flags8(&mut self, register: u8, value: u8) {
        self.registers
            .set_half_carry((register & 0x0F) < (value & 0x0F));
        self.registers.set_carry(register < value);
    }

    fn set_add_carry_flags16(&mut self, register: u16, value: u16) {
        self.registers
            .set_half_carry((register & 0x0FFF) + (value & 0x0FFF) > 0x0FFF);
        self.registers
            .set_carry((register as u32) + (value as u32) > 0xFFFF);
    }

    fn set_add_carry_flags_with_carry_in(&mut self, register: u8, value: u8, carry: u8) {
        let has_half_carry = (register & 0x0F) + (value & 0x0F) + carry > 0x0F;
        let has_carry = (register as u16) + (value as u16) + (carry as u16) > 0xFF;

        self.registers.set_half_carry(has_half_carry);
        self.registers.set_carry(has_carry);
    }

    fn set_sub_borrow_flags_with_borrow_in(&mut self, register: u8, value: u8, borrow: u8) {
        let has_half_borrow = (register & 0x0F) < (value & 0x0F) + borrow;
        let has_borrow = (register as u16) < (value as u16) + (borrow as u16);

        self.registers.set_half_carry(has_half_borrow);
        self.registers.set_carry(has_borrow);
    }

    fn set_and_flags(&mut self, value: u8) {
        self.registers.set_zero(value == 0);
        self.registers.set_subtract(false);
        self.registers.set_half_carry(true);
        self.registers.set_carry(false);
    }

    fn set_or_flags(&mut self, value: u8) {
        self.registers.set_zero(value == 0);
        self.registers.set_subtract(false);
        self.registers.set_half_carry(false);
        self.registers.set_carry(false);
    }

    pub fn execute_arithmetic(&mut self, instruction: ArithmeticInstruction) {
        match instruction {
            ArithmeticInstruction::Inc(register) => {
                let register_value = self.get_register8(&register);
                let value = self.increment_8bit(register_value);

                self.set_register8(&register, value);
            }
            ArithmeticInstruction::Dec(register) => {
                let register_value = self.get_register8(&register);
                let value = self.decrement_8bit(register_value);

                self.set_register8(&register, value);
            }
            ArithmeticInstruction::Inc16(register) => {
                let value = self.get_register16(&register);
                self.set_register16(&register, value.wrapping_add(1));
            }
            ArithmeticInstruction::Dec16(register) => {
                let value = self.get_register16(&register);
                self.set_register16(&register, value.wrapping_sub(1));
            }
            ArithmeticInstruction::AddHl(register) => {
                let hl = self.registers.get_hl();
                let register_value = self.get_register16(&register);

                let value = hl.wrapping_add(register_value);

                self.registers.set_subtract(false);
                self.set_add_carry_flags16(hl, register_value);

                self.registers.set_hl(value);
            }
            ArithmeticInstruction::AddSpImmediate => {
                let sp = self.registers.get_sp();
                let offset = self.fetch() as i8;
                let value = (sp as i16 + offset as i16) as u16;

                self.set_flags_sp_plus_immediate(sp, offset);
                self.registers.set_sp(value);
            }

            ArithmeticInstruction::Add(register) => {
                let a = self.registers.get_a();
                let value = self.get_register8(&register);

                let new_a = a.wrapping_add(value);

                self.registers.set_zero(new_a == 0);
                self.registers.set_subtract(false);
                self.set_add_carry_flags8(a, value);

                self.registers.set_a(new_a);
            }
            ArithmeticInstruction::AddFromHl => {
                let hl = self.registers.get_hl();
                let a = self.registers.get_a();

                let value = self.bus.read(hl);

                let new_a = a.wrapping_add(value);

                self.registers.set_zero(new_a == 0);
                self.registers.set_subtract(false);
                self.set_add_carry_flags8(a, value);

                self.registers.set_a(new_a);
            }
            ArithmeticInstruction::AddImmediate => {
                let a = self.registers.get_a();
                let value = self.fetch();

                let new_a = a.wrapping_add(value);

                self.registers.set_zero(new_a == 0);
                self.registers.set_subtract(false);
                self.set_add_carry_flags8(a, value);

                self.registers.set_a(new_a);
            }
            ArithmeticInstruction::Adc(register) => {
                let a = self.registers.get_a();
                let value = self.get_register8(&register);
                let carry = self.registers.get_carry() as u8;

                let new_a = a.wrapping_add(value).wrapping_add(carry);

                self.registers.set_zero(new_a == 0);
                self.registers.set_subtract(false);
                self.set_add_carry_flags_with_carry_in(a, value, carry);

                self.registers.set_a(new_a);
            }
            ArithmeticInstruction::AdcFromHl => {
                let a = self.registers.get_a();
                let hl = self.registers.get_hl();
                let carry = self.registers.get_carry() as u8;

                let value = self.bus.read(hl);

                let new_a = a.wrapping_add(value).wrapping_add(carry);

                self.registers.set_zero(new_a == 0);
                self.registers.set_subtract(false);
                self.set_add_carry_flags_with_carry_in(a, value, carry);

                self.registers.set_a(new_a);
            }
            ArithmeticInstruction::AdcImmediate => {
                let a = self.registers.get_a();
                let value = self.fetch();
                let carry = self.registers.get_carry() as u8;

                let new_a = a.wrapping_add(value).wrapping_add(carry);

                self.registers.set_zero(new_a == 0);
                self.registers.set_subtract(false);
                self.set_add_carry_flags_with_carry_in(a, value, carry);

                self.registers.set_a(new_a);
            }
            ArithmeticInstruction::Sub(register) => {
                let a = self.registers.get_a();
                let value = self.get_register8(&register);

                let new_a = a.wrapping_sub(value);

                self.registers.set_zero(new_a == 0);
                self.registers.set_subtract(true);
                self.set_sub_borrow_flags8(a, value);

                self.registers.set_a(new_a);
            }
            ArithmeticInstruction::SubFromHl => {
                let hl = self.registers.get_hl();
                let a = self.registers.get_a();

                let value = self.bus.read(hl);

                let new_a = a.wrapping_sub(value);

                self.registers.set_zero(new_a == 0);
                self.registers.set_subtract(true);
                self.set_sub_borrow_flags8(a, value);

                self.registers.set_a(new_a);
            }
            ArithmeticInstruction::SubImmediate => {
                let a = self.registers.get_a();
                let value = self.fetch();

                let new_a = a.wrapping_sub(value);

                self.registers.set_zero(new_a == 0);
                self.registers.set_subtract(true);
                self.set_sub_borrow_flags8(a, value);

                self.registers.set_a(new_a);
            }
            ArithmeticInstruction::Sbc(register) => {
                let a = self.registers.get_a();
                let value = self.get_register8(&register);
                let borrow = self.registers.get_carry() as u8;

                let new_a = a.wrapping_sub(value).wrapping_sub(borrow);

                self.registers.set_zero(new_a == 0);
                self.registers.set_subtract(true);
                self.set_sub_borrow_flags_with_borrow_in(a, value, borrow);

                self.registers.set_a(new_a);
            }
            ArithmeticInstruction::SbcFromHl => {
                let a = self.registers.get_a();
                let hl = self.registers.get_hl();
                let borrow = self.registers.get_carry() as u8;

                let value = self.bus.read(hl);

                let new_a = a.wrapping_sub(value).wrapping_sub(borrow);

                self.registers.set_zero(new_a == 0);
                self.registers.set_subtract(true);
                self.set_sub_borrow_flags_with_borrow_in(a, value, borrow);

                self.registers.set_a(new_a);
            }
            ArithmeticInstruction::SbcImmediate => {
                let a = self.registers.get_a();
                let value = self.fetch();
                let borrow = self.registers.get_carry() as u8;

                let new_a = a.wrapping_sub(value).wrapping_sub(borrow);

                self.registers.set_zero(new_a == 0);
                self.registers.set_subtract(true);
                self.set_sub_borrow_flags_with_borrow_in(a, value, borrow);

                self.registers.set_a(new_a);
            }
            ArithmeticInstruction::And(register) => {
                let a = self.registers.get_a();
                let value = self.get_register8(&register) & a;

                self.set_and_flags(value);
                self.registers.set_a(value);
            }
            ArithmeticInstruction::AndFromHl => {
                let a = self.registers.get_a();
                let hl = self.registers.get_hl();

                let value = self.bus.read(hl) & a;

                self.set_and_flags(value);
                self.registers.set_a(value);
            }
            ArithmeticInstruction::AndImmediate => {
                let a = self.registers.get_a();
                let value = self.fetch() & a;

                self.set_and_flags(value);
                self.registers.set_a(value);
            }
            ArithmeticInstruction::Or(register) => {
                let a = self.registers.get_a();
                let value = self.get_register8(&register) | a;

                self.set_or_flags(value);
                self.registers.set_a(value);
            }
            ArithmeticInstruction::OrFromHl => {
                let a = self.registers.get_a();
                let hl = self.registers.get_hl();

                let value = self.bus.read(hl) | a;

                self.set_or_flags(value);
                self.registers.set_a(value);
            }
            ArithmeticInstruction::OrImmediate => {
                let a = self.registers.get_a();
                let value = self.fetch() | a;

                self.set_or_flags(value);
                self.registers.set_a(value);
            }
        }
    }
}
