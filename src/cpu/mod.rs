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

    fn set_carry_flags8(&mut self, register: u8, value: u8) {
        self.registers
            .set_half_carry((register & 0x0F) + (value & 0x0F) > 0x0F);
        self.registers
            .set_carry((register as u16) + (value as u16) > 0xFF);
    }

    fn set_carry_flags16(&mut self, register: u16, value: u16) {
        self.registers
            .set_half_carry((register & 0x0FFF) + (value & 0x0FFF) > 0x0FFF);
        self.registers
            .set_carry((register as u32) + (value as u32) > 0xFFFF);
    }

    fn set_flags_sp_plus_immediate(&mut self, sp: u16, offset: i8) {
        let offset = offset as u8;

        self.registers.set_zero(false);
        self.registers.set_subtract(false);
        self.registers
            .set_half_carry((sp & 0x000F) + (offset as u16 & 0x000F) > 0x000F);

        self.registers
            .set_carry((sp & 0x00FF) + offset as u16 > 0x00FF);
    }

    fn set_carry_flags_with_carry_in(&mut self, register: u8, value: u8, carry: u8) {
        let has_half_carry = (register & 0x0F) + (value & 0x0F) + carry > 0x0F;
        let has_carry = (register as u16) + (value as u16) + (carry as u16) > 0xFF;

        self.registers.set_half_carry(has_half_carry);
        self.registers.set_carry(has_carry);
    }

    fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::Nop => {}
            Instruction::Inc(register) => {
                let register_value = self.get_register8(&register);
                let value = self.increment_8bit(register_value);

                self.set_register8(&register, value);
            }
            Instruction::Dec(register) => {
                let register_value = self.get_register8(&register);
                let value = self.decrement_8bit(register_value);

                self.set_register8(&register, value);
            }
            Instruction::Load8Immediate(register) => {
                let value = self.fetch();
                self.set_register8(&register, value);
            }
            Instruction::Load8Register(register_destination, register_source) => {
                let value = self.get_register8(&register_source);
                self.set_register8(&register_destination, value);
            }
            Instruction::Load8FromHl(register) => {
                let address = self.registers.get_hl();
                let value = self.bus.read(address);

                self.set_register8(&register, value);
            }
            Instruction::Load8ToHl(register) => {
                let address = self.registers.get_hl();
                let value = self.get_register8(&register);

                self.bus.write(address, value);
            }
            Instruction::Load8ToHlImmediate => {
                let address = self.registers.get_hl();
                let value = self.fetch();

                self.bus.write(address, value);
            }
            Instruction::Load8FromBc => {
                let address = self.registers.get_bc();
                let value = self.bus.read(address);

                self.registers.set_a(value);
            }
            Instruction::Load8FromDe => {
                let address = self.registers.get_de();
                let value = self.bus.read(address);

                self.registers.set_a(value);
            }
            Instruction::Load8ToBc => {
                let address = self.registers.get_bc();
                let value = self.registers.get_a();

                self.bus.write(address, value);
            }
            Instruction::Load8ToDe => {
                let address = self.registers.get_de();
                let value = self.registers.get_a();

                self.bus.write(address, value);
            }
            Instruction::Load8FromAddress => {
                let lower_bits = self.fetch();
                let higher_bits = self.fetch();

                let address = ((higher_bits as u16) << 8) | lower_bits as u16;
                let value = self.bus.read(address);

                self.registers.set_a(value);
            }
            Instruction::Load8ToAddress => {
                let lower_bits = self.fetch();
                let higher_bits = self.fetch();

                let address = ((higher_bits as u16) << 8) | lower_bits as u16;
                let value = self.registers.get_a();

                self.bus.write(address, value);
            }
            Instruction::Load16Immediate(register) => {
                let lower_bits = self.fetch();
                let higher_bits = self.fetch();

                let value = ((higher_bits as u16) << 8) | lower_bits as u16;

                self.set_register16(&register, value);
            }
            Instruction::Load16ToAddress(register) => {
                let lower_address = self.fetch();
                let higher_address = self.fetch();

                let address = ((higher_address as u16) << 8) | lower_address as u16;
                let value = self.get_register16(&register);

                let lower_value = value as u8;
                let higher_value = (value >> 8) as u8;

                self.bus.write(address, lower_value);
                self.bus.write(address + 1, higher_value);
            }
            Instruction::LoadSpFromHl => {
                let value = self.registers.get_hl();
                self.registers.set_sp(value);
            }
            Instruction::LoadHlFromSpPlusImmediate => {
                let sp = self.registers.get_sp();
                let offset = self.fetch() as i8;
                let value = (sp as i16 + offset as i16) as u16;

                self.set_flags_sp_plus_immediate(sp, offset);

                self.registers.set_hl(value);
            }
            Instruction::Inc16(register) => {
                let value = self.get_register16(&register);
                self.set_register16(&register, value.wrapping_add(1));
            }
            Instruction::Dec16(register) => {
                let value = self.get_register16(&register);
                self.set_register16(&register, value.wrapping_sub(1));
            }
            Instruction::AddHl(register) => {
                let hl = self.registers.get_hl();
                let register_value = self.get_register16(&register);

                let value = hl.wrapping_add(register_value);

                self.registers.set_subtract(false);
                self.set_carry_flags16(hl, register_value);

                self.registers.set_hl(value);
            }
            Instruction::AddSpImmediate => {
                let sp = self.registers.get_sp();
                let offset = self.fetch() as i8;
                let value = (sp as i16 + offset as i16) as u16;

                self.set_flags_sp_plus_immediate(sp, offset);
                self.registers.set_sp(value);
            }
            Instruction::Push(register) => {
                let sp = self.registers.get_sp();
                let new_sp = sp.wrapping_sub(2);

                let value = self.get_stack_register(&register);
                let lower_value = value as u8;
                let higher_value = (value >> 8) as u8;

                self.bus.write(sp.wrapping_sub(1), higher_value);
                self.bus.write(new_sp, lower_value);

                self.registers.set_sp(new_sp);
            }
            Instruction::Pop(register) => {
                let sp = self.registers.get_sp();
                let new_sp = sp.wrapping_add(2);

                let lower_value = self.bus.read(sp);
                let higher_value = self.bus.read(sp.wrapping_add(1));

                let value = u16::from_le_bytes([lower_value, higher_value]);

                self.set_stack_register(&register, value);
                self.registers.set_sp(new_sp);
            }
            Instruction::Add(register) => {
                let a = self.registers.get_a();
                let value = self.get_register8(&register);

                let new_a = a.wrapping_add(value);

                self.registers.set_zero(new_a == 0);
                self.registers.set_subtract(false);
                self.set_carry_flags8(a, value);

                self.registers.set_a(new_a);
            }
            Instruction::AddFromHl => {
                let hl = self.registers.get_hl();
                let a = self.registers.get_a();

                let value = self.bus.read(hl);

                let new_a = a.wrapping_add(value);

                self.registers.set_zero(new_a == 0);
                self.registers.set_subtract(false);
                self.set_carry_flags8(a, value);

                self.registers.set_a(new_a);
            }
            Instruction::AddImmediate => {
                let a = self.registers.get_a();
                let value = self.fetch();

                let new_a = a.wrapping_add(value);

                self.registers.set_zero(new_a == 0);
                self.registers.set_subtract(false);
                self.set_carry_flags8(a, value);

                self.registers.set_a(new_a);
            }
            Instruction::Adc(register) => {
                let a = self.registers.get_a();
                let value = self.get_register8(&register);
                let carry = self.registers.get_carry() as u8;

                let new_a = a.wrapping_add(value).wrapping_add(carry);

                self.registers.set_zero(new_a == 0);
                self.registers.set_subtract(false);
                self.set_carry_flags_with_carry_in(a, value, carry);

                self.registers.set_a(new_a);
            }
            Instruction::AdcFromHl => {
                let a = self.registers.get_a();
                let hl = self.registers.get_hl();
                let carry = self.registers.get_carry() as u8;

                let value = self.bus.read(hl);

                let new_a = a.wrapping_add(value).wrapping_add(carry);

                self.registers.set_zero(new_a == 0);
                self.registers.set_subtract(false);
                self.set_carry_flags_with_carry_in(a, value, carry);

                self.registers.set_a(new_a);
            }
            Instruction::AdcImmediate => {
                let a = self.registers.get_a();
                let value = self.fetch();
                let carry = self.registers.get_carry() as u8;

                let new_a = a.wrapping_add(value).wrapping_add(carry);

                self.registers.set_zero(new_a == 0);
                self.registers.set_subtract(false);
                self.set_carry_flags_with_carry_in(a, value, carry);

                self.registers.set_a(new_a);
            }
        }
    }

    pub fn step(&mut self) {
        let opcode = self.fetch();
        let instruction = decode(opcode);

        self.execute(instruction);
    }
}
