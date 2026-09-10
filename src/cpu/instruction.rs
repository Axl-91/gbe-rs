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
    Dec(Register8),
    Load8(Register8),
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

        0x05 => Instruction::Dec(Register8::B),
        0x0D => Instruction::Dec(Register8::C),
        0x15 => Instruction::Dec(Register8::D),
        0x1D => Instruction::Dec(Register8::E),
        0x25 => Instruction::Dec(Register8::H),
        0x2D => Instruction::Dec(Register8::L),
        0x3D => Instruction::Dec(Register8::A),

        0x06 => Instruction::Load8(Register8::B),
        0x0E => Instruction::Load8(Register8::C),
        0x16 => Instruction::Load8(Register8::D),
        0x1E => Instruction::Load8(Register8::E),
        0x26 => Instruction::Load8(Register8::H),
        0x2E => Instruction::Load8(Register8::L),
        0x3E => Instruction::Load8(Register8::A),
        _ => panic!("Unknown opcode: {opcode:#04X}"),
    }
}
