#[derive(Clone, Copy)]
pub enum Register8 {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

pub enum Register16 {
    BC,
    DE,
    HL,
    SP,
}

pub enum StackRegister {
    AF,
    BC,
    DE,
    HL,
}

pub enum Instruction {
    Nop,

    Load(LoadInstruction),

    Arithmetic(ArithmeticInstruction),

    Stack(StackInstruction),

    #[allow(dead_code)]
    Control(ControlInstruction),
}

pub enum LoadInstruction {
    Load8Immediate(Register8),
    Load8Register(Register8, Register8),
    Load8FromHl(Register8),
    Load8ToHl(Register8),
    Load8ToHlImmediate,
    Load8FromBc,
    Load8FromDe,
    Load8ToBc,
    Load8ToDe,
    Load8FromAddress,
    Load8ToAddress,
    Load16Immediate(Register16),
    Load16ToAddress(Register16),
    LoadSpFromHl,
    LoadHlFromSpPlusImmediate,
}

pub enum ArithmeticInstruction {
    Inc(Register8),
    Dec(Register8),

    Inc16(Register16),
    Dec16(Register16),
    AddHl(Register16),
    AddSpImmediate,

    Add(Register8),
    AddFromHl,
    AddImmediate,

    Adc(Register8),
    AdcFromHl,
    AdcImmediate,

    Sub(Register8),
    SubFromHl,
    SubImmediate,

    Sbc(Register8),
    SbcFromHl,
    SbcImmediate,

    And(Register8),
    AndFromHl,
    AndImmediate,

    Or(Register8),
    OrFromHl,
    OrImmediate,
}

pub enum StackInstruction {
    Push(StackRegister),
    Pop(StackRegister),
}

pub enum ControlInstruction {
    // JR
    // JP
    // CALL
    // RET
    // RST
    // HALT
    // STOP
    // DI
    // EI
}

pub fn decode(opcode: u8) -> Instruction {
    match opcode {
        // NOP
        0x00 => Instruction::Nop,

        // INC r
        0x04 => Instruction::Arithmetic(ArithmeticInstruction::Inc(Register8::B)),
        0x0C => Instruction::Arithmetic(ArithmeticInstruction::Inc(Register8::C)),
        0x14 => Instruction::Arithmetic(ArithmeticInstruction::Inc(Register8::D)),
        0x1C => Instruction::Arithmetic(ArithmeticInstruction::Inc(Register8::E)),
        0x24 => Instruction::Arithmetic(ArithmeticInstruction::Inc(Register8::H)),
        0x2C => Instruction::Arithmetic(ArithmeticInstruction::Inc(Register8::L)),
        0x3C => Instruction::Arithmetic(ArithmeticInstruction::Inc(Register8::A)),

        // DEC r
        0x05 => Instruction::Arithmetic(ArithmeticInstruction::Dec(Register8::B)),
        0x0D => Instruction::Arithmetic(ArithmeticInstruction::Dec(Register8::C)),
        0x15 => Instruction::Arithmetic(ArithmeticInstruction::Dec(Register8::D)),
        0x1D => Instruction::Arithmetic(ArithmeticInstruction::Dec(Register8::E)),
        0x25 => Instruction::Arithmetic(ArithmeticInstruction::Dec(Register8::H)),
        0x2D => Instruction::Arithmetic(ArithmeticInstruction::Dec(Register8::L)),
        0x3D => Instruction::Arithmetic(ArithmeticInstruction::Dec(Register8::A)),

        // LD r, n
        0x06 => Instruction::Load(LoadInstruction::Load8Immediate(Register8::B)),
        0x0E => Instruction::Load(LoadInstruction::Load8Immediate(Register8::C)),
        0x16 => Instruction::Load(LoadInstruction::Load8Immediate(Register8::D)),
        0x1E => Instruction::Load(LoadInstruction::Load8Immediate(Register8::E)),
        0x26 => Instruction::Load(LoadInstruction::Load8Immediate(Register8::H)),
        0x2E => Instruction::Load(LoadInstruction::Load8Immediate(Register8::L)),
        0x3E => Instruction::Load(LoadInstruction::Load8Immediate(Register8::A)),

        // LD r, r
        0x40 => Instruction::Load(LoadInstruction::Load8Register(Register8::B, Register8::B)),
        0x41 => Instruction::Load(LoadInstruction::Load8Register(Register8::B, Register8::C)),
        0x42 => Instruction::Load(LoadInstruction::Load8Register(Register8::B, Register8::D)),
        0x43 => Instruction::Load(LoadInstruction::Load8Register(Register8::B, Register8::E)),
        0x44 => Instruction::Load(LoadInstruction::Load8Register(Register8::B, Register8::H)),
        0x45 => Instruction::Load(LoadInstruction::Load8Register(Register8::B, Register8::L)),
        0x47 => Instruction::Load(LoadInstruction::Load8Register(Register8::B, Register8::A)),

        0x48 => Instruction::Load(LoadInstruction::Load8Register(Register8::C, Register8::B)),
        0x49 => Instruction::Load(LoadInstruction::Load8Register(Register8::C, Register8::C)),
        0x4A => Instruction::Load(LoadInstruction::Load8Register(Register8::C, Register8::D)),
        0x4B => Instruction::Load(LoadInstruction::Load8Register(Register8::C, Register8::E)),
        0x4C => Instruction::Load(LoadInstruction::Load8Register(Register8::C, Register8::H)),
        0x4D => Instruction::Load(LoadInstruction::Load8Register(Register8::C, Register8::L)),
        0x4F => Instruction::Load(LoadInstruction::Load8Register(Register8::C, Register8::A)),

        0x50 => Instruction::Load(LoadInstruction::Load8Register(Register8::D, Register8::B)),
        0x51 => Instruction::Load(LoadInstruction::Load8Register(Register8::D, Register8::C)),
        0x52 => Instruction::Load(LoadInstruction::Load8Register(Register8::D, Register8::D)),
        0x53 => Instruction::Load(LoadInstruction::Load8Register(Register8::D, Register8::E)),
        0x54 => Instruction::Load(LoadInstruction::Load8Register(Register8::D, Register8::H)),
        0x55 => Instruction::Load(LoadInstruction::Load8Register(Register8::D, Register8::L)),
        0x57 => Instruction::Load(LoadInstruction::Load8Register(Register8::D, Register8::A)),

        0x58 => Instruction::Load(LoadInstruction::Load8Register(Register8::E, Register8::B)),
        0x59 => Instruction::Load(LoadInstruction::Load8Register(Register8::E, Register8::C)),
        0x5A => Instruction::Load(LoadInstruction::Load8Register(Register8::E, Register8::D)),
        0x5B => Instruction::Load(LoadInstruction::Load8Register(Register8::E, Register8::E)),
        0x5C => Instruction::Load(LoadInstruction::Load8Register(Register8::E, Register8::H)),
        0x5D => Instruction::Load(LoadInstruction::Load8Register(Register8::E, Register8::L)),
        0x5F => Instruction::Load(LoadInstruction::Load8Register(Register8::E, Register8::A)),

        0x60 => Instruction::Load(LoadInstruction::Load8Register(Register8::H, Register8::B)),
        0x61 => Instruction::Load(LoadInstruction::Load8Register(Register8::H, Register8::C)),
        0x62 => Instruction::Load(LoadInstruction::Load8Register(Register8::H, Register8::D)),
        0x63 => Instruction::Load(LoadInstruction::Load8Register(Register8::H, Register8::E)),
        0x64 => Instruction::Load(LoadInstruction::Load8Register(Register8::H, Register8::H)),
        0x65 => Instruction::Load(LoadInstruction::Load8Register(Register8::H, Register8::L)),
        0x67 => Instruction::Load(LoadInstruction::Load8Register(Register8::H, Register8::A)),

        0x68 => Instruction::Load(LoadInstruction::Load8Register(Register8::L, Register8::B)),
        0x69 => Instruction::Load(LoadInstruction::Load8Register(Register8::L, Register8::C)),
        0x6A => Instruction::Load(LoadInstruction::Load8Register(Register8::L, Register8::D)),
        0x6B => Instruction::Load(LoadInstruction::Load8Register(Register8::L, Register8::E)),
        0x6C => Instruction::Load(LoadInstruction::Load8Register(Register8::L, Register8::H)),
        0x6D => Instruction::Load(LoadInstruction::Load8Register(Register8::L, Register8::L)),
        0x6F => Instruction::Load(LoadInstruction::Load8Register(Register8::L, Register8::A)),

        0x78 => Instruction::Load(LoadInstruction::Load8Register(Register8::A, Register8::B)),
        0x79 => Instruction::Load(LoadInstruction::Load8Register(Register8::A, Register8::C)),
        0x7A => Instruction::Load(LoadInstruction::Load8Register(Register8::A, Register8::D)),
        0x7B => Instruction::Load(LoadInstruction::Load8Register(Register8::A, Register8::E)),
        0x7C => Instruction::Load(LoadInstruction::Load8Register(Register8::A, Register8::H)),
        0x7D => Instruction::Load(LoadInstruction::Load8Register(Register8::A, Register8::L)),
        0x7F => Instruction::Load(LoadInstruction::Load8Register(Register8::A, Register8::A)),

        // LD r, (HL)
        0x46 => Instruction::Load(LoadInstruction::Load8FromHl(Register8::B)),
        0x4E => Instruction::Load(LoadInstruction::Load8FromHl(Register8::C)),
        0x56 => Instruction::Load(LoadInstruction::Load8FromHl(Register8::D)),
        0x5E => Instruction::Load(LoadInstruction::Load8FromHl(Register8::E)),
        0x66 => Instruction::Load(LoadInstruction::Load8FromHl(Register8::H)),
        0x6E => Instruction::Load(LoadInstruction::Load8FromHl(Register8::L)),
        0x7E => Instruction::Load(LoadInstruction::Load8FromHl(Register8::A)),

        // LD (HL), r
        0x70 => Instruction::Load(LoadInstruction::Load8ToHl(Register8::B)),
        0x71 => Instruction::Load(LoadInstruction::Load8ToHl(Register8::C)),
        0x72 => Instruction::Load(LoadInstruction::Load8ToHl(Register8::D)),
        0x73 => Instruction::Load(LoadInstruction::Load8ToHl(Register8::E)),
        0x74 => Instruction::Load(LoadInstruction::Load8ToHl(Register8::H)),
        0x75 => Instruction::Load(LoadInstruction::Load8ToHl(Register8::L)),
        0x77 => Instruction::Load(LoadInstruction::Load8ToHl(Register8::A)),

        // LD (HL), n
        0x36 => Instruction::Load(LoadInstruction::Load8ToHlImmediate),

        // LD A, (BC)
        0x0A => Instruction::Load(LoadInstruction::Load8FromBc),

        // LD A, (DE)
        0x1A => Instruction::Load(LoadInstruction::Load8FromDe),

        // LD (BC), A
        0x02 => Instruction::Load(LoadInstruction::Load8ToBc),

        // LD (DE), A
        0x12 => Instruction::Load(LoadInstruction::Load8ToDe),

        // LD A, (a16)
        0xFA => Instruction::Load(LoadInstruction::Load8FromAddress),

        // LD (a16), A
        0xEA => Instruction::Load(LoadInstruction::Load8ToAddress),

        // LD rr, nn
        0x01 => Instruction::Load(LoadInstruction::Load16Immediate(Register16::BC)),
        0x11 => Instruction::Load(LoadInstruction::Load16Immediate(Register16::DE)),
        0x21 => Instruction::Load(LoadInstruction::Load16Immediate(Register16::HL)),
        0x31 => Instruction::Load(LoadInstruction::Load16Immediate(Register16::SP)),

        // LD (a16), SP
        0x08 => Instruction::Load(LoadInstruction::Load16ToAddress(Register16::SP)),

        // LD HL, SP + e8
        0xF8 => Instruction::Load(LoadInstruction::LoadHlFromSpPlusImmediate),

        // LD SP, HL
        0xF9 => Instruction::Load(LoadInstruction::LoadSpFromHl),

        // INC rr
        0x03 => Instruction::Arithmetic(ArithmeticInstruction::Inc16(Register16::BC)),
        0x13 => Instruction::Arithmetic(ArithmeticInstruction::Inc16(Register16::DE)),
        0x23 => Instruction::Arithmetic(ArithmeticInstruction::Inc16(Register16::HL)),
        0x33 => Instruction::Arithmetic(ArithmeticInstruction::Inc16(Register16::SP)),

        // DEC rr
        0x0B => Instruction::Arithmetic(ArithmeticInstruction::Dec16(Register16::BC)),
        0x1B => Instruction::Arithmetic(ArithmeticInstruction::Dec16(Register16::DE)),
        0x2B => Instruction::Arithmetic(ArithmeticInstruction::Dec16(Register16::HL)),
        0x3B => Instruction::Arithmetic(ArithmeticInstruction::Dec16(Register16::SP)),

        // ADD HL, rr
        0x09 => Instruction::Arithmetic(ArithmeticInstruction::AddHl(Register16::BC)),
        0x19 => Instruction::Arithmetic(ArithmeticInstruction::AddHl(Register16::DE)),
        0x29 => Instruction::Arithmetic(ArithmeticInstruction::AddHl(Register16::HL)),
        0x39 => Instruction::Arithmetic(ArithmeticInstruction::AddHl(Register16::SP)),

        // ADD SP, e8
        0xE8 => Instruction::Arithmetic(ArithmeticInstruction::AddSpImmediate),

        // PUSH rr
        0xC5 => Instruction::Stack(StackInstruction::Push(StackRegister::BC)),
        0xD5 => Instruction::Stack(StackInstruction::Push(StackRegister::DE)),
        0xE5 => Instruction::Stack(StackInstruction::Push(StackRegister::HL)),
        0xF5 => Instruction::Stack(StackInstruction::Push(StackRegister::AF)),

        // POP rr
        0xC1 => Instruction::Stack(StackInstruction::Pop(StackRegister::BC)),
        0xD1 => Instruction::Stack(StackInstruction::Pop(StackRegister::DE)),
        0xE1 => Instruction::Stack(StackInstruction::Pop(StackRegister::HL)),
        0xF1 => Instruction::Stack(StackInstruction::Pop(StackRegister::AF)),

        // ADD A, r
        0x80 => Instruction::Arithmetic(ArithmeticInstruction::Add(Register8::B)),
        0x81 => Instruction::Arithmetic(ArithmeticInstruction::Add(Register8::C)),
        0x82 => Instruction::Arithmetic(ArithmeticInstruction::Add(Register8::D)),
        0x83 => Instruction::Arithmetic(ArithmeticInstruction::Add(Register8::E)),
        0x84 => Instruction::Arithmetic(ArithmeticInstruction::Add(Register8::H)),
        0x85 => Instruction::Arithmetic(ArithmeticInstruction::Add(Register8::L)),
        0x87 => Instruction::Arithmetic(ArithmeticInstruction::Add(Register8::A)),

        // ADD A, (HL)
        0x86 => Instruction::Arithmetic(ArithmeticInstruction::AddFromHl),

        // ADD A, n
        0xC6 => Instruction::Arithmetic(ArithmeticInstruction::AddImmediate),

        // ADC A, r
        0x8F => Instruction::Arithmetic(ArithmeticInstruction::Adc(Register8::A)),
        0x88 => Instruction::Arithmetic(ArithmeticInstruction::Adc(Register8::B)),
        0x89 => Instruction::Arithmetic(ArithmeticInstruction::Adc(Register8::C)),
        0x8A => Instruction::Arithmetic(ArithmeticInstruction::Adc(Register8::D)),
        0x8B => Instruction::Arithmetic(ArithmeticInstruction::Adc(Register8::E)),
        0x8C => Instruction::Arithmetic(ArithmeticInstruction::Adc(Register8::H)),
        0x8D => Instruction::Arithmetic(ArithmeticInstruction::Adc(Register8::L)),

        // ADC A, (HL)
        0x8E => Instruction::Arithmetic(ArithmeticInstruction::AdcFromHl),

        // ADC A, n
        0xCE => Instruction::Arithmetic(ArithmeticInstruction::AdcImmediate),

        // SUB A, r
        0x97 => Instruction::Arithmetic(ArithmeticInstruction::Sub(Register8::A)),
        0x90 => Instruction::Arithmetic(ArithmeticInstruction::Sub(Register8::B)),
        0x91 => Instruction::Arithmetic(ArithmeticInstruction::Sub(Register8::C)),
        0x92 => Instruction::Arithmetic(ArithmeticInstruction::Sub(Register8::D)),
        0x93 => Instruction::Arithmetic(ArithmeticInstruction::Sub(Register8::E)),
        0x94 => Instruction::Arithmetic(ArithmeticInstruction::Sub(Register8::H)),
        0x95 => Instruction::Arithmetic(ArithmeticInstruction::Sub(Register8::L)),

        // SUB A, (HL)
        0x96 => Instruction::Arithmetic(ArithmeticInstruction::SubFromHl),

        // SUB A, n
        0xD6 => Instruction::Arithmetic(ArithmeticInstruction::SubImmediate),

        // SBC A, r
        0x9F => Instruction::Arithmetic(ArithmeticInstruction::Sbc(Register8::A)),
        0x98 => Instruction::Arithmetic(ArithmeticInstruction::Sbc(Register8::B)),
        0x99 => Instruction::Arithmetic(ArithmeticInstruction::Sbc(Register8::C)),
        0x9A => Instruction::Arithmetic(ArithmeticInstruction::Sbc(Register8::D)),
        0x9B => Instruction::Arithmetic(ArithmeticInstruction::Sbc(Register8::E)),
        0x9C => Instruction::Arithmetic(ArithmeticInstruction::Sbc(Register8::H)),
        0x9D => Instruction::Arithmetic(ArithmeticInstruction::Sbc(Register8::L)),

        // SBC A, (HL)
        0x9E => Instruction::Arithmetic(ArithmeticInstruction::SbcFromHl),

        // SBC A, n
        0xDE => Instruction::Arithmetic(ArithmeticInstruction::SbcImmediate),

        // AND A, r
        0xA7 => Instruction::Arithmetic(ArithmeticInstruction::And(Register8::A)),
        0xA0 => Instruction::Arithmetic(ArithmeticInstruction::And(Register8::B)),
        0xA1 => Instruction::Arithmetic(ArithmeticInstruction::And(Register8::C)),
        0xA2 => Instruction::Arithmetic(ArithmeticInstruction::And(Register8::D)),
        0xA3 => Instruction::Arithmetic(ArithmeticInstruction::And(Register8::E)),
        0xA4 => Instruction::Arithmetic(ArithmeticInstruction::And(Register8::H)),
        0xA5 => Instruction::Arithmetic(ArithmeticInstruction::And(Register8::L)),

        // AND A, (HL)
        0xA6 => Instruction::Arithmetic(ArithmeticInstruction::AndFromHl),

        // AND A, n
        0xE6 => Instruction::Arithmetic(ArithmeticInstruction::AndImmediate),

        // OR A, r
        0xB7 => Instruction::Arithmetic(ArithmeticInstruction::Or(Register8::A)),
        0xB0 => Instruction::Arithmetic(ArithmeticInstruction::Or(Register8::B)),
        0xB1 => Instruction::Arithmetic(ArithmeticInstruction::Or(Register8::C)),
        0xB2 => Instruction::Arithmetic(ArithmeticInstruction::Or(Register8::D)),
        0xB3 => Instruction::Arithmetic(ArithmeticInstruction::Or(Register8::E)),
        0xB4 => Instruction::Arithmetic(ArithmeticInstruction::Or(Register8::H)),
        0xB5 => Instruction::Arithmetic(ArithmeticInstruction::Or(Register8::L)),

        // OR A, r
        0xB6 => Instruction::Arithmetic(ArithmeticInstruction::OrFromHl),

        // OR A, r
        0xF6 => Instruction::Arithmetic(ArithmeticInstruction::OrImmediate),

        _ => panic!("Unknown opcode: {opcode:#04X}"),
    }
}
