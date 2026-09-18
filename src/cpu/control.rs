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
    pub(crate) fn execute_control(&mut self, instruction: ControlInstruction) -> u8 {
        match instruction {
            ControlInstruction::Jr(condition) => {
                let offset = self.fetch() as i8 as i16;

                if self.should_i_jump(condition) {
                    let pc = self.registers.get_pc();

                    self.tick_internal();
                    self.registers.set_pc(pc.wrapping_add_signed(offset));

                    instruction.t_cycles_conditional(true)
                } else {
                    instruction.t_cycles_conditional(false)
                }
            }
            ControlInstruction::Jp(condition) => {
                let low_bits = self.fetch();
                let high_bits = self.fetch();

                let new_pc = u16::from_le_bytes([low_bits, high_bits]);

                if self.should_i_jump(condition) {
                    self.tick_internal();
                    self.registers.set_pc(new_pc);

                    instruction.t_cycles_conditional(true)
                } else {
                    instruction.t_cycles_conditional(false)
                }
            }
            ControlInstruction::Call(condition) => {
                let low_bits = self.fetch();
                let high_bits = self.fetch();
                let new_pc = u16::from_le_bytes([low_bits, high_bits]);

                if self.should_i_jump(condition) {
                    let pc = self.registers.get_pc();

                    self.tick_internal();
                    self.push_into_sp(pc);
                    self.registers.set_pc(new_pc);

                    instruction.t_cycles_conditional(true)
                } else {
                    instruction.t_cycles_conditional(false)
                }
            }
            ControlInstruction::Ret(condition) => {
                self.tick_internal();

                if self.should_i_jump(condition) {
                    let new_pc = self.pop_from_sp();

                    self.tick_internal();
                    self.registers.set_pc(new_pc);

                    instruction.t_cycles_conditional(true)
                } else {
                    instruction.t_cycles_conditional(false)
                }
            }
            ControlInstruction::Rst(code) => {
                let pc = self.registers.get_pc();

                self.tick_internal();
                self.push_into_sp(pc);
                self.registers.set_pc(code as u16);

                instruction.t_cycles()
            }
            ControlInstruction::JpHl => {
                let hl = self.registers.get_hl();
                self.registers.set_pc(hl);
                instruction.t_cycles()
            }
            ControlInstruction::Stop => {
                let pc = self.registers.get_pc();
                self.bus.read(pc);
                self.registers.set_pc(pc.wrapping_add(1));

                let joypad_active = self.bus.is_joypad_active();
                let interrupt_pending = self.check_interruption().is_some();

                if joypad_active || interrupt_pending {
                    // TODO: There's still logic to do here
                } else {
                    self.stopped = true;
                }
                instruction.t_cycles()
            }
            ControlInstruction::Halt => {
                self.halted = true;

                if !self.ime && self.check_interruption().is_some() {
                    self.halted = false;
                    self.halt_bug = true;
                }

                instruction.t_cycles()
            }
            ControlInstruction::DisableInterrupts => {
                self.ime = false;
                self.ime_schedule = false;
                instruction.t_cycles()
            }
            ControlInstruction::EnableInterrupts => {
                self.ime_schedule = true;
                instruction.t_cycles()
            }
            ControlInstruction::Reti => {
                let new_pc = self.pop_from_sp();

                self.tick_internal();
                self.registers.set_pc(new_pc);
                self.ime = true;

                instruction.t_cycles()
            }
        }
    }
}
