//! Game Boy CPU control flow instructions.
//!
//! Implements jumps, relative jumps, calls, returns, and restart instructions.

use crate::cpu::{
    Cpu,
    instruction::{Condition, ControlInstruction},
};

impl Cpu {
    fn should_i_jump(&self, condition: Option<Condition>) -> bool {
        match condition {
            None => true,
            Some(Condition::NotZero) => !self.registers.get_zero(),
            Some(Condition::Zero) => self.registers.get_zero(),
            Some(Condition::NotCarry) => !self.registers.get_carry(),
            Some(Condition::Carry) => self.registers.get_carry(),
        }
    }

    /// Executes a CPU control flow instruction.
    pub(crate) fn execute_control(&mut self, instruction: ControlInstruction) {
        match instruction {
            ControlInstruction::Jr(condition) => {
                let offset = self.fetch() as i8 as i16;

                if self.should_i_jump(condition) {
                    let pc = self.registers.get_pc();
                    self.registers.set_pc(pc.wrapping_add_signed(offset));
                }
            }
            ControlInstruction::Jp(condition) => {
                let low_bits = self.fetch();
                let high_bits = self.fetch();

                let new_pc = u16::from_le_bytes([low_bits, high_bits]);

                if self.should_i_jump(condition) {
                    self.registers.set_pc(new_pc);
                }
            }
            ControlInstruction::Call(condition) => {
                let low_bits = self.fetch();
                let high_bits = self.fetch();
                let new_pc = u16::from_le_bytes([low_bits, high_bits]);

                if self.should_i_jump(condition) {
                    let pc = self.registers.get_pc();
                    self.push_into_sp(pc);
                    self.registers.set_pc(new_pc);
                }
            }
            ControlInstruction::Ret(condition) => {
                if self.should_i_jump(condition) {
                    let new_pc = self.pop_from_sp();
                    self.registers.set_pc(new_pc);
                }
            }
            ControlInstruction::Rst(code) => {
                let pc = self.registers.get_pc();
                self.push_into_sp(pc);
                self.registers.set_pc(code as u16);
            }
            ControlInstruction::JpHl => {
                let hl = self.registers.get_hl();
                self.registers.set_pc(hl);
            }
            ControlInstruction::Stop => {
                todo!("Once interruptions are implemented we add the logic")
            }
            ControlInstruction::Halt => {
                todo!("Once interruptions are implemented we add the logic")
            }
            ControlInstruction::DisableInterrupts => {
                todo!("Once interruptions are implemented we add the logic")
            }
            ControlInstruction::EnableInterrupts => {
                todo!("Once interruptions are implemented we add the logic")
            }
            ControlInstruction::Reti => {
                todo!("Once interruptions are implemented we add the logic")
            }
        }
    }
}
