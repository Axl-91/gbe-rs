use crate::cpu::{Cpu, instruction::ControlInstruction};

impl Cpu {
    pub fn execute_control(&mut self, instruction: ControlInstruction) {
        match instruction {}
    }
}
