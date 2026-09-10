#[derive(Debug)]
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

    // INC r
    Inc(Register8),

    // DEC r
    Dec(Register8),

    // LD r, n
    Load8Immediate(Register8),

    // LD r, r
    Load8Register(Register8, Register8),

    // LD r, (hl)
    Load8FromHl(Register8),

    //LD (hl), r
    Load8ToHl(Register8),

    // LD (hl), n
    Load8ToHlImmediate,
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

        0x06 => Instruction::Load8Immediate(Register8::B),
        0x0E => Instruction::Load8Immediate(Register8::C),
        0x16 => Instruction::Load8Immediate(Register8::D),
        0x1E => Instruction::Load8Immediate(Register8::E),
        0x26 => Instruction::Load8Immediate(Register8::H),
        0x2E => Instruction::Load8Immediate(Register8::L),
        0x3E => Instruction::Load8Immediate(Register8::A),

        0x40 => Instruction::Load8Register(Register8::B, Register8::B),
        0x41 => Instruction::Load8Register(Register8::B, Register8::C),
        0x42 => Instruction::Load8Register(Register8::B, Register8::D),
        0x43 => Instruction::Load8Register(Register8::B, Register8::E),
        0x44 => Instruction::Load8Register(Register8::B, Register8::H),
        0x45 => Instruction::Load8Register(Register8::B, Register8::L),
        0x47 => Instruction::Load8Register(Register8::B, Register8::A),

        0x48 => Instruction::Load8Register(Register8::C, Register8::B),
        0x49 => Instruction::Load8Register(Register8::C, Register8::C),
        0x4A => Instruction::Load8Register(Register8::C, Register8::D),
        0x4B => Instruction::Load8Register(Register8::C, Register8::E),
        0x4C => Instruction::Load8Register(Register8::C, Register8::H),
        0x4D => Instruction::Load8Register(Register8::C, Register8::L),
        0x4F => Instruction::Load8Register(Register8::C, Register8::A),

        0x50 => Instruction::Load8Register(Register8::D, Register8::B),
        0x51 => Instruction::Load8Register(Register8::D, Register8::C),
        0x52 => Instruction::Load8Register(Register8::D, Register8::D),
        0x53 => Instruction::Load8Register(Register8::D, Register8::E),
        0x54 => Instruction::Load8Register(Register8::D, Register8::H),
        0x55 => Instruction::Load8Register(Register8::D, Register8::L),
        0x57 => Instruction::Load8Register(Register8::D, Register8::A),

        0x58 => Instruction::Load8Register(Register8::E, Register8::B),
        0x59 => Instruction::Load8Register(Register8::E, Register8::C),
        0x5A => Instruction::Load8Register(Register8::E, Register8::D),
        0x5B => Instruction::Load8Register(Register8::E, Register8::E),
        0x5C => Instruction::Load8Register(Register8::E, Register8::H),
        0x5D => Instruction::Load8Register(Register8::E, Register8::L),
        0x5F => Instruction::Load8Register(Register8::E, Register8::A),

        0x60 => Instruction::Load8Register(Register8::H, Register8::B),
        0x61 => Instruction::Load8Register(Register8::H, Register8::C),
        0x62 => Instruction::Load8Register(Register8::H, Register8::D),
        0x63 => Instruction::Load8Register(Register8::H, Register8::E),
        0x64 => Instruction::Load8Register(Register8::H, Register8::H),
        0x65 => Instruction::Load8Register(Register8::H, Register8::L),
        0x67 => Instruction::Load8Register(Register8::H, Register8::A),

        0x68 => Instruction::Load8Register(Register8::L, Register8::B),
        0x69 => Instruction::Load8Register(Register8::L, Register8::C),
        0x6A => Instruction::Load8Register(Register8::L, Register8::D),
        0x6B => Instruction::Load8Register(Register8::L, Register8::E),
        0x6C => Instruction::Load8Register(Register8::L, Register8::H),
        0x6D => Instruction::Load8Register(Register8::L, Register8::L),
        0x6F => Instruction::Load8Register(Register8::L, Register8::A),

        0x78 => Instruction::Load8Register(Register8::A, Register8::B),
        0x79 => Instruction::Load8Register(Register8::A, Register8::C),
        0x7A => Instruction::Load8Register(Register8::A, Register8::D),
        0x7B => Instruction::Load8Register(Register8::A, Register8::E),
        0x7C => Instruction::Load8Register(Register8::A, Register8::H),
        0x7D => Instruction::Load8Register(Register8::A, Register8::L),
        0x7F => Instruction::Load8Register(Register8::A, Register8::A),

        0x46 => Instruction::Load8FromHl(Register8::B),
        0x4E => Instruction::Load8FromHl(Register8::C),
        0x56 => Instruction::Load8FromHl(Register8::D),
        0x5E => Instruction::Load8FromHl(Register8::E),
        0x66 => Instruction::Load8FromHl(Register8::H),
        0x6E => Instruction::Load8FromHl(Register8::L),
        0x7E => Instruction::Load8FromHl(Register8::A),

        0x70 => Instruction::Load8ToHl(Register8::B),
        0x71 => Instruction::Load8ToHl(Register8::C),
        0x72 => Instruction::Load8ToHl(Register8::D),
        0x73 => Instruction::Load8ToHl(Register8::E),
        0x74 => Instruction::Load8ToHl(Register8::H),
        0x75 => Instruction::Load8ToHl(Register8::L),
        0x77 => Instruction::Load8ToHl(Register8::A),

        0x36 => Instruction::Load8ToHlImmediate,

        _ => panic!("Unknown opcode: {opcode:#04X}"),
    }
}
