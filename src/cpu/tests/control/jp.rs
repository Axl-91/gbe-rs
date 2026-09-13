use super::*;

#[test]
fn jp() {
    let mut rng = rand::rng();

    let target = rng.random_range(u16::MIN..=u16::MAX);
    let [low, high] = target.to_le_bytes();

    let mut cpu = create_cpu(0xC3, Some(low), Some(high));

    cpu.step();

    assert_eq!(cpu.registers.get_pc(), target);
}

#[test]
fn jp_conditions() {
    let cases = [
        (0xC2, Condition::NotZero),
        (0xCA, Condition::Zero),
        (0xD2, Condition::NotCarry),
        (0xDA, Condition::Carry),
    ];

    let mut rng = rand::rng();

    for (opcode, condition) in cases {
        let target = rng.random_range(u16::MIN..=u16::MAX);
        let [low, high] = target.to_le_bytes();

        let mut cpu = create_cpu(opcode, Some(low), Some(high));

        match condition {
            Condition::NotZero => cpu.registers.set_zero(false),
            Condition::Zero => cpu.registers.set_zero(true),
            Condition::NotCarry => cpu.registers.set_carry(false),
            Condition::Carry => cpu.registers.set_carry(true),
        }

        cpu.step();

        assert_eq!(cpu.registers.get_pc(), target);
    }
}

#[test]
fn jp_condition_not_met() {
    let cases = [
        (0xC2, Condition::NotZero),
        (0xCA, Condition::Zero),
        (0xD2, Condition::NotCarry),
        (0xDA, Condition::Carry),
    ];

    let mut rng = rand::rng();

    for (opcode, condition) in cases {
        let target = rng.random_range(u16::MIN..=u16::MAX);
        let [low, high] = target.to_le_bytes();

        let mut cpu = create_cpu(opcode, Some(low), Some(high));

        match condition {
            Condition::NotZero => cpu.registers.set_zero(true),
            Condition::Zero => cpu.registers.set_zero(false),
            Condition::NotCarry => cpu.registers.set_carry(true),
            Condition::Carry => cpu.registers.set_carry(false),
        }

        cpu.step();

        assert_eq!(cpu.registers.get_pc(), GAME_ENTRY_POINT + 3);
    }
}
