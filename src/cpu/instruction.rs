//! Game Boy CPU instruction definitions.
//!
//! Defines the instruction types used to represent decoded CPU operations.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Register8 {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Register16 {
    BC,
    DE,
    HL,
    SP,
}

#[derive(Debug, PartialEq, Eq)]
pub enum StackRegister {
    AF,
    BC,
    DE,
    HL,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Condition {
    NotZero,
    Zero,
    NotCarry,
    Carry,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Instruction {
    Nop,

    Load(LoadInstruction),

    Arithmetic(ArithmeticInstruction),

    Stack(StackInstruction),

    Rotation(RotationInstruction),

    Control(ControlInstruction),

    Cb,
}

#[derive(Debug, PartialEq, Eq)]
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

    Load8ToHighAddress,
    Load8FromHighAddress,
    Load8ToHighRegister,
    Load8FromHighRegister,

    Load8ToAddressHlIncrement,
    Load8ToAddressHlDecrement,
    Load8FromAddressHlIncrement,
    Load8FromAddressHlDecrement,
}

#[derive(Debug, PartialEq, Eq)]
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

    Xor(Register8),
    XorFromHl,
    XorImmediate,

    Cp(Register8),
    CpFromHl,
    CpImmediate,

    Daa,
    Cpl,
    Scf,
    Ccf,

    IncFromHl,
    DecFromHl,
}

#[derive(Debug, PartialEq, Eq)]
pub enum StackInstruction {
    Push(StackRegister),
    Pop(StackRegister),
}

#[derive(Debug, PartialEq, Eq)]
pub enum RotationInstruction {
    Rlca,
    Rla,
    Rrca,
    Rra,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ControlInstruction {
    Jr(Option<Condition>),
    Jp(Option<Condition>),
    Call(Option<Condition>),
    Ret(Option<Condition>),
    Rst(u8),
    JpHl,
    Stop,
    Halt,
    DisableInterrupts,
    EnableInterrupts,
    Reti,
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

        // OR A, (HL)
        0xB6 => Instruction::Arithmetic(ArithmeticInstruction::OrFromHl),

        // OR A, n
        0xF6 => Instruction::Arithmetic(ArithmeticInstruction::OrImmediate),

        // XOR A, r
        0xAF => Instruction::Arithmetic(ArithmeticInstruction::Xor(Register8::A)),
        0xA8 => Instruction::Arithmetic(ArithmeticInstruction::Xor(Register8::B)),
        0xA9 => Instruction::Arithmetic(ArithmeticInstruction::Xor(Register8::C)),
        0xAA => Instruction::Arithmetic(ArithmeticInstruction::Xor(Register8::D)),
        0xAB => Instruction::Arithmetic(ArithmeticInstruction::Xor(Register8::E)),
        0xAC => Instruction::Arithmetic(ArithmeticInstruction::Xor(Register8::H)),
        0xAD => Instruction::Arithmetic(ArithmeticInstruction::Xor(Register8::L)),

        // XOR A, (HL)
        0xAE => Instruction::Arithmetic(ArithmeticInstruction::XorFromHl),

        // XOR A, n
        0xEE => Instruction::Arithmetic(ArithmeticInstruction::XorImmediate),

        // CP A, r
        0xBF => Instruction::Arithmetic(ArithmeticInstruction::Cp(Register8::A)),
        0xB8 => Instruction::Arithmetic(ArithmeticInstruction::Cp(Register8::B)),
        0xB9 => Instruction::Arithmetic(ArithmeticInstruction::Cp(Register8::C)),
        0xBA => Instruction::Arithmetic(ArithmeticInstruction::Cp(Register8::D)),
        0xBB => Instruction::Arithmetic(ArithmeticInstruction::Cp(Register8::E)),
        0xBC => Instruction::Arithmetic(ArithmeticInstruction::Cp(Register8::H)),
        0xBD => Instruction::Arithmetic(ArithmeticInstruction::Cp(Register8::L)),

        // CP A, (HL)
        0xBE => Instruction::Arithmetic(ArithmeticInstruction::CpFromHl),

        // CP A, n
        0xFE => Instruction::Arithmetic(ArithmeticInstruction::CpImmediate),

        // DAA
        0x27 => Instruction::Arithmetic(ArithmeticInstruction::Daa),
        // CPL
        0x2F => Instruction::Arithmetic(ArithmeticInstruction::Cpl),
        // SCF
        0x37 => Instruction::Arithmetic(ArithmeticInstruction::Scf),
        // CCF
        0x3F => Instruction::Arithmetic(ArithmeticInstruction::Ccf),

        // RLCA
        0x07 => Instruction::Rotation(RotationInstruction::Rlca),
        // RLA
        0x17 => Instruction::Rotation(RotationInstruction::Rla),
        // RRCA
        0x0F => Instruction::Rotation(RotationInstruction::Rrca),
        // RRA
        0x1F => Instruction::Rotation(RotationInstruction::Rra),

        // JR -/Z/NZ/C/NC
        0x18 => Instruction::Control(ControlInstruction::Jr(None)),
        0x20 => Instruction::Control(ControlInstruction::Jr(Some(Condition::NotZero))),
        0x28 => Instruction::Control(ControlInstruction::Jr(Some(Condition::Zero))),
        0x30 => Instruction::Control(ControlInstruction::Jr(Some(Condition::NotCarry))),
        0x38 => Instruction::Control(ControlInstruction::Jr(Some(Condition::Carry))),

        // JP -/Z/NZ/C/NC
        0xC3 => Instruction::Control(ControlInstruction::Jp(None)),
        0xC2 => Instruction::Control(ControlInstruction::Jp(Some(Condition::NotZero))),
        0xCA => Instruction::Control(ControlInstruction::Jp(Some(Condition::Zero))),
        0xD2 => Instruction::Control(ControlInstruction::Jp(Some(Condition::NotCarry))),
        0xDA => Instruction::Control(ControlInstruction::Jp(Some(Condition::Carry))),

        // CALL -/Z/NZ/C/NC
        0xCD => Instruction::Control(ControlInstruction::Call(None)),
        0xC4 => Instruction::Control(ControlInstruction::Call(Some(Condition::NotZero))),
        0xCC => Instruction::Control(ControlInstruction::Call(Some(Condition::Zero))),
        0xD4 => Instruction::Control(ControlInstruction::Call(Some(Condition::NotCarry))),
        0xDC => Instruction::Control(ControlInstruction::Call(Some(Condition::Carry))),

        // RET -/Z/NZ/C/NC
        0xC9 => Instruction::Control(ControlInstruction::Ret(None)),
        0xC0 => Instruction::Control(ControlInstruction::Ret(Some(Condition::NotZero))),
        0xC8 => Instruction::Control(ControlInstruction::Ret(Some(Condition::Zero))),
        0xD0 => Instruction::Control(ControlInstruction::Ret(Some(Condition::NotCarry))),
        0xD8 => Instruction::Control(ControlInstruction::Ret(Some(Condition::Carry))),

        // RST u8
        0xC7 => Instruction::Control(ControlInstruction::Rst(0x00)),
        0xCF => Instruction::Control(ControlInstruction::Rst(0x08)),
        0xD7 => Instruction::Control(ControlInstruction::Rst(0x10)),
        0xDF => Instruction::Control(ControlInstruction::Rst(0x18)),
        0xE7 => Instruction::Control(ControlInstruction::Rst(0x20)),
        0xEF => Instruction::Control(ControlInstruction::Rst(0x28)),
        0xF7 => Instruction::Control(ControlInstruction::Rst(0x30)),
        0xFF => Instruction::Control(ControlInstruction::Rst(0x38)),

        // JP HL
        0xE9 => Instruction::Control(ControlInstruction::JpHl),

        // STOP
        0x10 => Instruction::Control(ControlInstruction::Stop),
        // HALT
        0x76 => Instruction::Control(ControlInstruction::Halt),
        // DI
        0xF3 => Instruction::Control(ControlInstruction::DisableInterrupts),
        // EI
        0xFB => Instruction::Control(ControlInstruction::EnableInterrupts),
        // RETI
        0xD9 => Instruction::Control(ControlInstruction::Reti),

        // LDH (a8), A
        0xE0 => Instruction::Load(LoadInstruction::Load8ToHighAddress),
        0xF0 => Instruction::Load(LoadInstruction::Load8FromHighAddress),
        // LDH (C), A
        0xE2 => Instruction::Load(LoadInstruction::Load8ToHighRegister),
        0xF2 => Instruction::Load(LoadInstruction::Load8FromHighRegister),

        // LD (HL+/-), A
        0x22 => Instruction::Load(LoadInstruction::Load8ToAddressHlIncrement),
        0x32 => Instruction::Load(LoadInstruction::Load8ToAddressHlDecrement),
        0x2A => Instruction::Load(LoadInstruction::Load8FromAddressHlIncrement),
        0x3A => Instruction::Load(LoadInstruction::Load8FromAddressHlDecrement),

        0x34 => Instruction::Arithmetic(ArithmeticInstruction::IncFromHl),
        0x35 => Instruction::Arithmetic(ArithmeticInstruction::DecFromHl),

        0xCB => Instruction::Cb,

        _ => panic!("Unknown opcode: {opcode:#04X}"),
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum CbRotation {
    Rlc(CbTarget),
    Rrc(CbTarget),
    Rl(CbTarget),
    Rr(CbTarget),
    Sla(CbTarget),
    Sra(CbTarget),
    Swap(CbTarget),
    Srl(CbTarget),
}

#[derive(Debug, PartialEq, Eq)]
pub enum CbTarget {
    Register(Register8),
    FromHl,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CbInstruction {
    Bit(u8, CbTarget),
    Res(u8, CbTarget),
    Set(u8, CbTarget),
    Rotation(CbRotation),
}

pub fn decode_cb(opcode: u8) -> CbInstruction {
    match opcode {
        // RLC
        0x00 => CbInstruction::Rotation(CbRotation::Rlc(CbTarget::Register(Register8::B))),
        0x01 => CbInstruction::Rotation(CbRotation::Rlc(CbTarget::Register(Register8::C))),
        0x02 => CbInstruction::Rotation(CbRotation::Rlc(CbTarget::Register(Register8::D))),
        0x03 => CbInstruction::Rotation(CbRotation::Rlc(CbTarget::Register(Register8::E))),
        0x04 => CbInstruction::Rotation(CbRotation::Rlc(CbTarget::Register(Register8::H))),
        0x05 => CbInstruction::Rotation(CbRotation::Rlc(CbTarget::Register(Register8::L))),
        0x06 => CbInstruction::Rotation(CbRotation::Rlc(CbTarget::FromHl)),
        0x07 => CbInstruction::Rotation(CbRotation::Rlc(CbTarget::Register(Register8::A))),

        // RRC
        0x08 => CbInstruction::Rotation(CbRotation::Rrc(CbTarget::Register(Register8::B))),
        0x09 => CbInstruction::Rotation(CbRotation::Rrc(CbTarget::Register(Register8::C))),
        0x0A => CbInstruction::Rotation(CbRotation::Rrc(CbTarget::Register(Register8::D))),
        0x0B => CbInstruction::Rotation(CbRotation::Rrc(CbTarget::Register(Register8::E))),
        0x0C => CbInstruction::Rotation(CbRotation::Rrc(CbTarget::Register(Register8::H))),
        0x0D => CbInstruction::Rotation(CbRotation::Rrc(CbTarget::Register(Register8::L))),
        0x0E => CbInstruction::Rotation(CbRotation::Rrc(CbTarget::FromHl)),
        0x0F => CbInstruction::Rotation(CbRotation::Rrc(CbTarget::Register(Register8::A))),

        // RL
        0x10 => CbInstruction::Rotation(CbRotation::Rl(CbTarget::Register(Register8::B))),
        0x11 => CbInstruction::Rotation(CbRotation::Rl(CbTarget::Register(Register8::C))),
        0x12 => CbInstruction::Rotation(CbRotation::Rl(CbTarget::Register(Register8::D))),
        0x13 => CbInstruction::Rotation(CbRotation::Rl(CbTarget::Register(Register8::E))),
        0x14 => CbInstruction::Rotation(CbRotation::Rl(CbTarget::Register(Register8::H))),
        0x15 => CbInstruction::Rotation(CbRotation::Rl(CbTarget::Register(Register8::L))),
        0x16 => CbInstruction::Rotation(CbRotation::Rl(CbTarget::FromHl)),
        0x17 => CbInstruction::Rotation(CbRotation::Rl(CbTarget::Register(Register8::A))),

        // RR
        0x18 => CbInstruction::Rotation(CbRotation::Rr(CbTarget::Register(Register8::B))),
        0x19 => CbInstruction::Rotation(CbRotation::Rr(CbTarget::Register(Register8::C))),
        0x1A => CbInstruction::Rotation(CbRotation::Rr(CbTarget::Register(Register8::D))),
        0x1B => CbInstruction::Rotation(CbRotation::Rr(CbTarget::Register(Register8::E))),
        0x1C => CbInstruction::Rotation(CbRotation::Rr(CbTarget::Register(Register8::H))),
        0x1D => CbInstruction::Rotation(CbRotation::Rr(CbTarget::Register(Register8::L))),
        0x1E => CbInstruction::Rotation(CbRotation::Rr(CbTarget::FromHl)),
        0x1F => CbInstruction::Rotation(CbRotation::Rr(CbTarget::Register(Register8::A))),

        // SLA
        0x20 => CbInstruction::Rotation(CbRotation::Sla(CbTarget::Register(Register8::B))),
        0x21 => CbInstruction::Rotation(CbRotation::Sla(CbTarget::Register(Register8::C))),
        0x22 => CbInstruction::Rotation(CbRotation::Sla(CbTarget::Register(Register8::D))),
        0x23 => CbInstruction::Rotation(CbRotation::Sla(CbTarget::Register(Register8::E))),
        0x24 => CbInstruction::Rotation(CbRotation::Sla(CbTarget::Register(Register8::H))),
        0x25 => CbInstruction::Rotation(CbRotation::Sla(CbTarget::Register(Register8::L))),
        0x26 => CbInstruction::Rotation(CbRotation::Sla(CbTarget::FromHl)),
        0x27 => CbInstruction::Rotation(CbRotation::Sla(CbTarget::Register(Register8::A))),

        // SRA
        0x28 => CbInstruction::Rotation(CbRotation::Sra(CbTarget::Register(Register8::B))),
        0x29 => CbInstruction::Rotation(CbRotation::Sra(CbTarget::Register(Register8::C))),
        0x2A => CbInstruction::Rotation(CbRotation::Sra(CbTarget::Register(Register8::D))),
        0x2B => CbInstruction::Rotation(CbRotation::Sra(CbTarget::Register(Register8::E))),
        0x2C => CbInstruction::Rotation(CbRotation::Sra(CbTarget::Register(Register8::H))),
        0x2D => CbInstruction::Rotation(CbRotation::Sra(CbTarget::Register(Register8::L))),
        0x2E => CbInstruction::Rotation(CbRotation::Sra(CbTarget::FromHl)),
        0x2F => CbInstruction::Rotation(CbRotation::Sra(CbTarget::Register(Register8::A))),

        // SWAP
        0x30 => CbInstruction::Rotation(CbRotation::Swap(CbTarget::Register(Register8::B))),
        0x31 => CbInstruction::Rotation(CbRotation::Swap(CbTarget::Register(Register8::C))),
        0x32 => CbInstruction::Rotation(CbRotation::Swap(CbTarget::Register(Register8::D))),
        0x33 => CbInstruction::Rotation(CbRotation::Swap(CbTarget::Register(Register8::E))),
        0x34 => CbInstruction::Rotation(CbRotation::Swap(CbTarget::Register(Register8::H))),
        0x35 => CbInstruction::Rotation(CbRotation::Swap(CbTarget::Register(Register8::L))),
        0x36 => CbInstruction::Rotation(CbRotation::Swap(CbTarget::FromHl)),
        0x37 => CbInstruction::Rotation(CbRotation::Swap(CbTarget::Register(Register8::A))),

        // SRL
        0x38 => CbInstruction::Rotation(CbRotation::Srl(CbTarget::Register(Register8::B))),
        0x39 => CbInstruction::Rotation(CbRotation::Srl(CbTarget::Register(Register8::C))),
        0x3A => CbInstruction::Rotation(CbRotation::Srl(CbTarget::Register(Register8::D))),
        0x3B => CbInstruction::Rotation(CbRotation::Srl(CbTarget::Register(Register8::E))),
        0x3C => CbInstruction::Rotation(CbRotation::Srl(CbTarget::Register(Register8::H))),
        0x3D => CbInstruction::Rotation(CbRotation::Srl(CbTarget::Register(Register8::L))),
        0x3E => CbInstruction::Rotation(CbRotation::Srl(CbTarget::FromHl)),
        0x3F => CbInstruction::Rotation(CbRotation::Srl(CbTarget::Register(Register8::A))),

        // BIT 0
        0x40 => CbInstruction::Bit(0, CbTarget::Register(Register8::B)),
        0x41 => CbInstruction::Bit(0, CbTarget::Register(Register8::C)),
        0x42 => CbInstruction::Bit(0, CbTarget::Register(Register8::D)),
        0x43 => CbInstruction::Bit(0, CbTarget::Register(Register8::E)),
        0x44 => CbInstruction::Bit(0, CbTarget::Register(Register8::H)),
        0x45 => CbInstruction::Bit(0, CbTarget::Register(Register8::L)),
        0x46 => CbInstruction::Bit(0, CbTarget::FromHl),
        0x47 => CbInstruction::Bit(0, CbTarget::Register(Register8::A)),

        // BIT 1
        0x48 => CbInstruction::Bit(1, CbTarget::Register(Register8::B)),
        0x49 => CbInstruction::Bit(1, CbTarget::Register(Register8::C)),
        0x4A => CbInstruction::Bit(1, CbTarget::Register(Register8::D)),
        0x4B => CbInstruction::Bit(1, CbTarget::Register(Register8::E)),
        0x4C => CbInstruction::Bit(1, CbTarget::Register(Register8::H)),
        0x4D => CbInstruction::Bit(1, CbTarget::Register(Register8::L)),
        0x4E => CbInstruction::Bit(1, CbTarget::FromHl),
        0x4F => CbInstruction::Bit(1, CbTarget::Register(Register8::A)),

        // BIT 2
        0x50 => CbInstruction::Bit(2, CbTarget::Register(Register8::B)),
        0x51 => CbInstruction::Bit(2, CbTarget::Register(Register8::C)),
        0x52 => CbInstruction::Bit(2, CbTarget::Register(Register8::D)),
        0x53 => CbInstruction::Bit(2, CbTarget::Register(Register8::E)),
        0x54 => CbInstruction::Bit(2, CbTarget::Register(Register8::H)),
        0x55 => CbInstruction::Bit(2, CbTarget::Register(Register8::L)),
        0x56 => CbInstruction::Bit(2, CbTarget::FromHl),
        0x57 => CbInstruction::Bit(2, CbTarget::Register(Register8::A)),

        // BIT 3
        0x58 => CbInstruction::Bit(3, CbTarget::Register(Register8::B)),
        0x59 => CbInstruction::Bit(3, CbTarget::Register(Register8::C)),
        0x5A => CbInstruction::Bit(3, CbTarget::Register(Register8::D)),
        0x5B => CbInstruction::Bit(3, CbTarget::Register(Register8::E)),
        0x5C => CbInstruction::Bit(3, CbTarget::Register(Register8::H)),
        0x5D => CbInstruction::Bit(3, CbTarget::Register(Register8::L)),
        0x5E => CbInstruction::Bit(3, CbTarget::FromHl),
        0x5F => CbInstruction::Bit(3, CbTarget::Register(Register8::A)),

        // BIT 4
        0x60 => CbInstruction::Bit(4, CbTarget::Register(Register8::B)),
        0x61 => CbInstruction::Bit(4, CbTarget::Register(Register8::C)),
        0x62 => CbInstruction::Bit(4, CbTarget::Register(Register8::D)),
        0x63 => CbInstruction::Bit(4, CbTarget::Register(Register8::E)),
        0x64 => CbInstruction::Bit(4, CbTarget::Register(Register8::H)),
        0x65 => CbInstruction::Bit(4, CbTarget::Register(Register8::L)),
        0x66 => CbInstruction::Bit(4, CbTarget::FromHl),
        0x67 => CbInstruction::Bit(4, CbTarget::Register(Register8::A)),

        // BIT 5
        0x68 => CbInstruction::Bit(5, CbTarget::Register(Register8::B)),
        0x69 => CbInstruction::Bit(5, CbTarget::Register(Register8::C)),
        0x6A => CbInstruction::Bit(5, CbTarget::Register(Register8::D)),
        0x6B => CbInstruction::Bit(5, CbTarget::Register(Register8::E)),
        0x6C => CbInstruction::Bit(5, CbTarget::Register(Register8::H)),
        0x6D => CbInstruction::Bit(5, CbTarget::Register(Register8::L)),
        0x6E => CbInstruction::Bit(5, CbTarget::FromHl),
        0x6F => CbInstruction::Bit(5, CbTarget::Register(Register8::A)),

        // BIT 6
        0x70 => CbInstruction::Bit(6, CbTarget::Register(Register8::B)),
        0x71 => CbInstruction::Bit(6, CbTarget::Register(Register8::C)),
        0x72 => CbInstruction::Bit(6, CbTarget::Register(Register8::D)),
        0x73 => CbInstruction::Bit(6, CbTarget::Register(Register8::E)),
        0x74 => CbInstruction::Bit(6, CbTarget::Register(Register8::H)),
        0x75 => CbInstruction::Bit(6, CbTarget::Register(Register8::L)),
        0x76 => CbInstruction::Bit(6, CbTarget::FromHl),
        0x77 => CbInstruction::Bit(6, CbTarget::Register(Register8::A)),

        // BIT 7
        0x78 => CbInstruction::Bit(7, CbTarget::Register(Register8::B)),
        0x79 => CbInstruction::Bit(7, CbTarget::Register(Register8::C)),
        0x7A => CbInstruction::Bit(7, CbTarget::Register(Register8::D)),
        0x7B => CbInstruction::Bit(7, CbTarget::Register(Register8::E)),
        0x7C => CbInstruction::Bit(7, CbTarget::Register(Register8::H)),
        0x7D => CbInstruction::Bit(7, CbTarget::Register(Register8::L)),
        0x7E => CbInstruction::Bit(7, CbTarget::FromHl),
        0x7F => CbInstruction::Bit(7, CbTarget::Register(Register8::A)),

        // RES 0
        0x80 => CbInstruction::Res(0, CbTarget::Register(Register8::B)),
        0x81 => CbInstruction::Res(0, CbTarget::Register(Register8::C)),
        0x82 => CbInstruction::Res(0, CbTarget::Register(Register8::D)),
        0x83 => CbInstruction::Res(0, CbTarget::Register(Register8::E)),
        0x84 => CbInstruction::Res(0, CbTarget::Register(Register8::H)),
        0x85 => CbInstruction::Res(0, CbTarget::Register(Register8::L)),
        0x86 => CbInstruction::Res(0, CbTarget::FromHl),
        0x87 => CbInstruction::Res(0, CbTarget::Register(Register8::A)),

        // RES 1
        0x88 => CbInstruction::Res(1, CbTarget::Register(Register8::B)),
        0x89 => CbInstruction::Res(1, CbTarget::Register(Register8::C)),
        0x8A => CbInstruction::Res(1, CbTarget::Register(Register8::D)),
        0x8B => CbInstruction::Res(1, CbTarget::Register(Register8::E)),
        0x8C => CbInstruction::Res(1, CbTarget::Register(Register8::H)),
        0x8D => CbInstruction::Res(1, CbTarget::Register(Register8::L)),
        0x8E => CbInstruction::Res(1, CbTarget::FromHl),
        0x8F => CbInstruction::Res(1, CbTarget::Register(Register8::A)),

        // RES 2
        0x90 => CbInstruction::Res(2, CbTarget::Register(Register8::B)),
        0x91 => CbInstruction::Res(2, CbTarget::Register(Register8::C)),
        0x92 => CbInstruction::Res(2, CbTarget::Register(Register8::D)),
        0x93 => CbInstruction::Res(2, CbTarget::Register(Register8::E)),
        0x94 => CbInstruction::Res(2, CbTarget::Register(Register8::H)),
        0x95 => CbInstruction::Res(2, CbTarget::Register(Register8::L)),
        0x96 => CbInstruction::Res(2, CbTarget::FromHl),
        0x97 => CbInstruction::Res(2, CbTarget::Register(Register8::A)),

        // RES 3
        0x98 => CbInstruction::Res(3, CbTarget::Register(Register8::B)),
        0x99 => CbInstruction::Res(3, CbTarget::Register(Register8::C)),
        0x9A => CbInstruction::Res(3, CbTarget::Register(Register8::D)),
        0x9B => CbInstruction::Res(3, CbTarget::Register(Register8::E)),
        0x9C => CbInstruction::Res(3, CbTarget::Register(Register8::H)),
        0x9D => CbInstruction::Res(3, CbTarget::Register(Register8::L)),
        0x9E => CbInstruction::Res(3, CbTarget::FromHl),
        0x9F => CbInstruction::Res(3, CbTarget::Register(Register8::A)),

        // RES 4
        0xA0 => CbInstruction::Res(4, CbTarget::Register(Register8::B)),
        0xA1 => CbInstruction::Res(4, CbTarget::Register(Register8::C)),
        0xA2 => CbInstruction::Res(4, CbTarget::Register(Register8::D)),
        0xA3 => CbInstruction::Res(4, CbTarget::Register(Register8::E)),
        0xA4 => CbInstruction::Res(4, CbTarget::Register(Register8::H)),
        0xA5 => CbInstruction::Res(4, CbTarget::Register(Register8::L)),
        0xA6 => CbInstruction::Res(4, CbTarget::FromHl),
        0xA7 => CbInstruction::Res(4, CbTarget::Register(Register8::A)),

        // RES 5
        0xA8 => CbInstruction::Res(5, CbTarget::Register(Register8::B)),
        0xA9 => CbInstruction::Res(5, CbTarget::Register(Register8::C)),
        0xAA => CbInstruction::Res(5, CbTarget::Register(Register8::D)),
        0xAB => CbInstruction::Res(5, CbTarget::Register(Register8::E)),
        0xAC => CbInstruction::Res(5, CbTarget::Register(Register8::H)),
        0xAD => CbInstruction::Res(5, CbTarget::Register(Register8::L)),
        0xAE => CbInstruction::Res(5, CbTarget::FromHl),
        0xAF => CbInstruction::Res(5, CbTarget::Register(Register8::A)),

        // RES 6
        0xB0 => CbInstruction::Res(6, CbTarget::Register(Register8::B)),
        0xB1 => CbInstruction::Res(6, CbTarget::Register(Register8::C)),
        0xB2 => CbInstruction::Res(6, CbTarget::Register(Register8::D)),
        0xB3 => CbInstruction::Res(6, CbTarget::Register(Register8::E)),
        0xB4 => CbInstruction::Res(6, CbTarget::Register(Register8::H)),
        0xB5 => CbInstruction::Res(6, CbTarget::Register(Register8::L)),
        0xB6 => CbInstruction::Res(6, CbTarget::FromHl),
        0xB7 => CbInstruction::Res(6, CbTarget::Register(Register8::A)),

        // RES 7
        0xB8 => CbInstruction::Res(7, CbTarget::Register(Register8::B)),
        0xB9 => CbInstruction::Res(7, CbTarget::Register(Register8::C)),
        0xBA => CbInstruction::Res(7, CbTarget::Register(Register8::D)),
        0xBB => CbInstruction::Res(7, CbTarget::Register(Register8::E)),
        0xBC => CbInstruction::Res(7, CbTarget::Register(Register8::H)),
        0xBD => CbInstruction::Res(7, CbTarget::Register(Register8::L)),
        0xBE => CbInstruction::Res(7, CbTarget::FromHl),
        0xBF => CbInstruction::Res(7, CbTarget::Register(Register8::A)),

        // SET 0
        0xC0 => CbInstruction::Set(0, CbTarget::Register(Register8::B)),
        0xC1 => CbInstruction::Set(0, CbTarget::Register(Register8::C)),
        0xC2 => CbInstruction::Set(0, CbTarget::Register(Register8::D)),
        0xC3 => CbInstruction::Set(0, CbTarget::Register(Register8::E)),
        0xC4 => CbInstruction::Set(0, CbTarget::Register(Register8::H)),
        0xC5 => CbInstruction::Set(0, CbTarget::Register(Register8::L)),
        0xC6 => CbInstruction::Set(0, CbTarget::FromHl),
        0xC7 => CbInstruction::Set(0, CbTarget::Register(Register8::A)),

        // SET 1
        0xC8 => CbInstruction::Set(1, CbTarget::Register(Register8::B)),
        0xC9 => CbInstruction::Set(1, CbTarget::Register(Register8::C)),
        0xCA => CbInstruction::Set(1, CbTarget::Register(Register8::D)),
        0xCB => CbInstruction::Set(1, CbTarget::Register(Register8::E)),
        0xCC => CbInstruction::Set(1, CbTarget::Register(Register8::H)),
        0xCD => CbInstruction::Set(1, CbTarget::Register(Register8::L)),
        0xCE => CbInstruction::Set(1, CbTarget::FromHl),
        0xCF => CbInstruction::Set(1, CbTarget::Register(Register8::A)),

        // SET 2
        0xD0 => CbInstruction::Set(2, CbTarget::Register(Register8::B)),
        0xD1 => CbInstruction::Set(2, CbTarget::Register(Register8::C)),
        0xD2 => CbInstruction::Set(2, CbTarget::Register(Register8::D)),
        0xD3 => CbInstruction::Set(2, CbTarget::Register(Register8::E)),
        0xD4 => CbInstruction::Set(2, CbTarget::Register(Register8::H)),
        0xD5 => CbInstruction::Set(2, CbTarget::Register(Register8::L)),
        0xD6 => CbInstruction::Set(2, CbTarget::FromHl),
        0xD7 => CbInstruction::Set(2, CbTarget::Register(Register8::A)),

        // SET 3
        0xD8 => CbInstruction::Set(3, CbTarget::Register(Register8::B)),
        0xD9 => CbInstruction::Set(3, CbTarget::Register(Register8::C)),
        0xDA => CbInstruction::Set(3, CbTarget::Register(Register8::D)),
        0xDB => CbInstruction::Set(3, CbTarget::Register(Register8::E)),
        0xDC => CbInstruction::Set(3, CbTarget::Register(Register8::H)),
        0xDD => CbInstruction::Set(3, CbTarget::Register(Register8::L)),
        0xDE => CbInstruction::Set(3, CbTarget::FromHl),
        0xDF => CbInstruction::Set(3, CbTarget::Register(Register8::A)),

        // SET 4
        0xE0 => CbInstruction::Set(4, CbTarget::Register(Register8::B)),
        0xE1 => CbInstruction::Set(4, CbTarget::Register(Register8::C)),
        0xE2 => CbInstruction::Set(4, CbTarget::Register(Register8::D)),
        0xE3 => CbInstruction::Set(4, CbTarget::Register(Register8::E)),
        0xE4 => CbInstruction::Set(4, CbTarget::Register(Register8::H)),
        0xE5 => CbInstruction::Set(4, CbTarget::Register(Register8::L)),
        0xE6 => CbInstruction::Set(4, CbTarget::FromHl),
        0xE7 => CbInstruction::Set(4, CbTarget::Register(Register8::A)),

        // SET 5
        0xE8 => CbInstruction::Set(5, CbTarget::Register(Register8::B)),
        0xE9 => CbInstruction::Set(5, CbTarget::Register(Register8::C)),
        0xEA => CbInstruction::Set(5, CbTarget::Register(Register8::D)),
        0xEB => CbInstruction::Set(5, CbTarget::Register(Register8::E)),
        0xEC => CbInstruction::Set(5, CbTarget::Register(Register8::H)),
        0xED => CbInstruction::Set(5, CbTarget::Register(Register8::L)),
        0xEE => CbInstruction::Set(5, CbTarget::FromHl),
        0xEF => CbInstruction::Set(5, CbTarget::Register(Register8::A)),

        // SET 6
        0xF0 => CbInstruction::Set(6, CbTarget::Register(Register8::B)),
        0xF1 => CbInstruction::Set(6, CbTarget::Register(Register8::C)),
        0xF2 => CbInstruction::Set(6, CbTarget::Register(Register8::D)),
        0xF3 => CbInstruction::Set(6, CbTarget::Register(Register8::E)),
        0xF4 => CbInstruction::Set(6, CbTarget::Register(Register8::H)),
        0xF5 => CbInstruction::Set(6, CbTarget::Register(Register8::L)),
        0xF6 => CbInstruction::Set(6, CbTarget::FromHl),
        0xF7 => CbInstruction::Set(6, CbTarget::Register(Register8::A)),

        // SET 7
        0xF8 => CbInstruction::Set(7, CbTarget::Register(Register8::B)),
        0xF9 => CbInstruction::Set(7, CbTarget::Register(Register8::C)),
        0xFA => CbInstruction::Set(7, CbTarget::Register(Register8::D)),
        0xFB => CbInstruction::Set(7, CbTarget::Register(Register8::E)),
        0xFC => CbInstruction::Set(7, CbTarget::Register(Register8::H)),
        0xFD => CbInstruction::Set(7, CbTarget::Register(Register8::L)),
        0xFE => CbInstruction::Set(7, CbTarget::FromHl),
        0xFF => CbInstruction::Set(7, CbTarget::Register(Register8::A)),
    }
}
