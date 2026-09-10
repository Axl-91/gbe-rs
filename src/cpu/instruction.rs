pub enum Register8 {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

pub enum Instruction {
    Nop,
    Inc(Register8),
}

pub fn decode(opcode: u8) -> Instruction {
    match opcode {
        0x00 => Instruction::Nop,
        0x04 => Instruction::Inc(Register8::B),
        0x0C => Instruction::Inc(Register8::C),
        0x14 => Instruction::Inc(Register8::D),
        0x1C => Instruction::Inc(Register8::E),
        0x24 => Instruction::Inc(Register8::H),
        0x2C => Instruction::Inc(Register8::L),
        0x3C => Instruction::Inc(Register8::A),
        _ => panic!("Unknown opcode: {opcode:#04X}"),
    }
}
