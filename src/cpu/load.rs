use crate::cpu::{Cpu, instruction::LoadInstruction};

impl Cpu {
    pub fn execute_load(&mut self, instruction: LoadInstruction) {
        match instruction {
            LoadInstruction::Load8Immediate(register) => {
                let value = self.fetch();
                self.set_register8(&register, value);
            }
            LoadInstruction::Load8Register(register_destination, register_source) => {
                let value = self.get_register8(&register_source);
                self.set_register8(&register_destination, value);
            }
            LoadInstruction::Load8FromHl(register) => {
                let address = self.registers.get_hl();
                let value = self.bus.read(address);

                self.set_register8(&register, value);
            }
            LoadInstruction::Load8ToHl(register) => {
                let address = self.registers.get_hl();
                let value = self.get_register8(&register);

                self.bus.write(address, value);
            }
            LoadInstruction::Load8ToHlImmediate => {
                let address = self.registers.get_hl();
                let value = self.fetch();

                self.bus.write(address, value);
            }
            LoadInstruction::Load8FromBc => {
                let address = self.registers.get_bc();
                let value = self.bus.read(address);

                self.registers.set_a(value);
            }
            LoadInstruction::Load8FromDe => {
                let address = self.registers.get_de();
                let value = self.bus.read(address);

                self.registers.set_a(value);
            }
            LoadInstruction::Load8ToBc => {
                let address = self.registers.get_bc();
                let value = self.registers.get_a();

                self.bus.write(address, value);
            }
            LoadInstruction::Load8ToDe => {
                let address = self.registers.get_de();
                let value = self.registers.get_a();

                self.bus.write(address, value);
            }
            LoadInstruction::Load8FromAddress => {
                let lower_bits = self.fetch();
                let higher_bits = self.fetch();

                let address = ((higher_bits as u16) << 8) | lower_bits as u16;
                let value = self.bus.read(address);

                self.registers.set_a(value);
            }
            LoadInstruction::Load8ToAddress => {
                let lower_bits = self.fetch();
                let higher_bits = self.fetch();

                let address = ((higher_bits as u16) << 8) | lower_bits as u16;
                let value = self.registers.get_a();

                self.bus.write(address, value);
            }
            LoadInstruction::Load16Immediate(register) => {
                let lower_bits = self.fetch();
                let higher_bits = self.fetch();

                let value = ((higher_bits as u16) << 8) | lower_bits as u16;

                self.set_register16(&register, value);
            }
            LoadInstruction::Load16ToAddress(register) => {
                let lower_address = self.fetch();
                let higher_address = self.fetch();

                let address = ((higher_address as u16) << 8) | lower_address as u16;
                let value = self.get_register16(&register);

                let lower_value = value as u8;
                let higher_value = (value >> 8) as u8;

                self.bus.write(address, lower_value);
                self.bus.write(address + 1, higher_value);
            }
            LoadInstruction::LoadSpFromHl => {
                let value = self.registers.get_hl();
                self.registers.set_sp(value);
            }
            LoadInstruction::LoadHlFromSpPlusImmediate => {
                let sp = self.registers.get_sp();
                let offset = self.fetch() as i8;
                let value = (sp as i16 + offset as i16) as u16;

                self.set_flags_sp_plus_immediate(sp, offset);

                self.registers.set_hl(value);
            }
        }
    }
}
