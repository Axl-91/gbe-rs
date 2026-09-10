pub enum Instruction {
    Nop,
    IncB,
}

pub fn decode(opcode: u8) -> Instruction {
    match opcode {
        0x00 => Instruction::Nop,
        0x04 => Instruction::IncB,
        _ => panic!("Unknown opcode: {opcode:#04X}"),
    }
}
