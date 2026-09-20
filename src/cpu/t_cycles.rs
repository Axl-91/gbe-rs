use crate::cpu::instruction::{
    ArithmeticInstruction, ControlInstruction, LoadInstruction, RotationInstruction,
    StackInstruction,
};

impl LoadInstruction {
    pub(crate) fn t_cycles(&self) -> u8 {
        match self {
            LoadInstruction::Load8Immediate(_) => 8,
            LoadInstruction::Load8Register(_, _) => 4,
            LoadInstruction::Load8FromHl(_) => 8,
            LoadInstruction::Load8ToHl(_) => 8,
            LoadInstruction::Load8ToHlImmediate => 12,
            LoadInstruction::Load8FromBc => 8,
            LoadInstruction::Load8FromDe => 8,
            LoadInstruction::Load8ToBc => 8,
            LoadInstruction::Load8ToDe => 8,
            LoadInstruction::Load8FromAddress => 16,
            LoadInstruction::Load8ToAddress => 16,

            LoadInstruction::Load16Immediate(_) => 12,
            LoadInstruction::Load16ToAddress(_) => 20,
            LoadInstruction::LoadSpFromHl => 8,
            LoadInstruction::LoadHlFromSpPlusImmediate => 12,

            LoadInstruction::Load8ToHighAddress => 12,
            LoadInstruction::Load8FromHighAddress => 12,
            LoadInstruction::Load8ToHighRegister => 8,
            LoadInstruction::Load8FromHighRegister => 8,

            LoadInstruction::Load8ToAddressHlIncrement => 8,
            LoadInstruction::Load8ToAddressHlDecrement => 8,
            LoadInstruction::Load8FromAddressHlIncrement => 8,
            LoadInstruction::Load8FromAddressHlDecrement => 8,
        }
    }
}

impl ArithmeticInstruction {
    pub(crate) fn t_cycles(&self) -> u8 {
        match self {
            ArithmeticInstruction::Inc(_) => 4,
            ArithmeticInstruction::Dec(_) => 4,

            ArithmeticInstruction::Inc16(_) => 8,
            ArithmeticInstruction::Dec16(_) => 8,
            ArithmeticInstruction::AddHl(_) => 8,
            ArithmeticInstruction::AddSpImmediate => 16,

            ArithmeticInstruction::Add(_) => 4,
            ArithmeticInstruction::AddFromHl => 8,
            ArithmeticInstruction::AddImmediate => 8,

            ArithmeticInstruction::Adc(_) => 4,
            ArithmeticInstruction::AdcFromHl => 8,
            ArithmeticInstruction::AdcImmediate => 8,

            ArithmeticInstruction::Sub(_) => 4,
            ArithmeticInstruction::SubFromHl => 8,
            ArithmeticInstruction::SubImmediate => 8,

            ArithmeticInstruction::Sbc(_) => 4,
            ArithmeticInstruction::SbcFromHl => 8,
            ArithmeticInstruction::SbcImmediate => 8,

            ArithmeticInstruction::And(_) => 4,
            ArithmeticInstruction::AndFromHl => 8,
            ArithmeticInstruction::AndImmediate => 8,

            ArithmeticInstruction::Or(_) => 4,
            ArithmeticInstruction::OrFromHl => 8,
            ArithmeticInstruction::OrImmediate => 8,

            ArithmeticInstruction::Xor(_) => 4,
            ArithmeticInstruction::XorFromHl => 8,
            ArithmeticInstruction::XorImmediate => 8,

            ArithmeticInstruction::Cp(_) => 4,
            ArithmeticInstruction::CpFromHl => 8,
            ArithmeticInstruction::CpImmediate => 8,

            ArithmeticInstruction::Daa => 4,
            ArithmeticInstruction::Cpl => 4,
            ArithmeticInstruction::Scf => 4,
            ArithmeticInstruction::Ccf => 4,

            ArithmeticInstruction::IncFromHl => 12,
            ArithmeticInstruction::DecFromHl => 12,
        }
    }
}

impl StackInstruction {
    pub(crate) fn t_cycles(&self) -> u8 {
        match self {
            StackInstruction::Push(_) => 16,
            StackInstruction::Pop(_) => 12,
        }
    }
}

impl RotationInstruction {
    pub(crate) fn t_cycles(&self) -> u8 {
        match self {
            RotationInstruction::Rlca => 4,
            RotationInstruction::Rla => 4,
            RotationInstruction::Rrca => 4,
            RotationInstruction::Rra => 4,
        }
    }
}

impl ControlInstruction {
    pub(crate) fn t_cycles(&self) -> u8 {
        match self {
            ControlInstruction::Rst(_) => 16,
            ControlInstruction::JpHl => 4,
            ControlInstruction::Stop => 4,
            ControlInstruction::Halt => 4,
            ControlInstruction::DisableInterrupts => 4,
            ControlInstruction::EnableInterrupts => 4,
            ControlInstruction::Reti => 16,
            ControlInstruction::Ret(None) => 16,

            _ => unreachable!("Conditional instruction requires a condition result"),
        }
    }

    pub(crate) fn t_cycles_conditional(&self, condition_met: bool) -> u8 {
        match self {
            ControlInstruction::Jr(_) => {
                if condition_met {
                    12
                } else {
                    8
                }
            }
            ControlInstruction::Jp(_) => {
                if condition_met {
                    16
                } else {
                    12
                }
            }
            ControlInstruction::Call(_) => {
                if condition_met {
                    24
                } else {
                    12
                }
            }
            ControlInstruction::Ret(_) => {
                if condition_met {
                    20
                } else {
                    8
                }
            }
            _ => unreachable!("Instruction is not conditional"),
        }
    }
}
