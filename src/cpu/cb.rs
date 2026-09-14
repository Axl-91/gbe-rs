use crate::cpu::{Cpu, instruction::CbInstruction};

impl Cpu {
    /// Executes a CB-prefixed instruction.
    pub(crate) fn execute_cb(&mut self, instruction: CbInstruction) {
        match instruction {
            CbInstruction::Bit(_value, _register) => {
                todo!()
            }
            CbInstruction::BitFromHl(_value) => {
                todo!()
            }
            CbInstruction::Res(_value, _register) => {
                todo!()
            }
            CbInstruction::ResFromHl(_value) => {
                todo!()
            }
            CbInstruction::Set(_value, _register) => {
                todo!()
            }
            CbInstruction::SetFromHl(_value) => {
                todo!()
            }
            CbInstruction::Rotation(instruction) => {
                self.execute_cb_rotation(instruction);
            }
        }
    }
}
