use super::*;

#[test]
fn jr() {
    let mut rng = rand::rng();

    let offset = rng.random_range(i8::MIN..=i8::MAX);
    let mut cpu = create_cpu(0x18, Some(offset as u8), None);

    cpu.step();

    let expected_pc = GAME_ENTRY_POINT
        .wrapping_add(2)
        .wrapping_add_signed(offset as i16);

    assert_eq!(cpu.registers.get_pc(), expected_pc);
}

#[test]
fn jr_conditions() {
    let cases = [
        (0x20, Condition::NotZero),
        (0x28, Condition::Zero),
        (0x30, Condition::NotCarry),
        (0x38, Condition::Carry),
    ];

    let mut rng = rand::rng();

    for (opcode, condition) in cases {
        let offset = rng.random_range(i8::MIN..=i8::MAX);
        let mut cpu = create_cpu(opcode, Some(offset as u8), None);

        match condition {
            Condition::NotZero => cpu.registers.set_zero(false),
            Condition::Zero => cpu.registers.set_zero(true),
            Condition::NotCarry => cpu.registers.set_carry(false),
            Condition::Carry => cpu.registers.set_carry(true),
        }

        cpu.step();

        let expected_pc = GAME_ENTRY_POINT
            .wrapping_add(2)
            .wrapping_add_signed(offset as i16);

        assert_eq!(cpu.registers.get_pc(), expected_pc);
    }
}

#[test]
fn jr_condition_not_met() {
    let cases = [
        (0x20, Condition::NotZero),
        (0x28, Condition::Zero),
        (0x30, Condition::NotCarry),
        (0x38, Condition::Carry),
    ];

    let mut rng = rand::rng();

    for (opcode, condition) in cases {
        let offset = rng.random_range(i8::MIN..=i8::MAX);
        let mut cpu = create_cpu(opcode, Some(offset as u8), None);

        match condition {
            Condition::NotZero => cpu.registers.set_zero(true),
            Condition::Zero => cpu.registers.set_zero(false),
            Condition::NotCarry => cpu.registers.set_carry(true),
            Condition::Carry => cpu.registers.set_carry(false),
        }

        cpu.step();

        assert_eq!(cpu.registers.get_pc(), GAME_ENTRY_POINT + 2);
    }
}
