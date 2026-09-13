use super::*;

#[test]
fn call() {
    let mut rng = rand::rng();

    let target = rng.random_range(u16::MIN..=u16::MAX);
    let [low, high] = target.to_le_bytes();

    let mut cpu = create_cpu(0xCD, Some(low), Some(high));

    let sp = get_rand_wram_address();
    cpu.registers.set_sp(sp);

    cpu.step();

    let expected_sp = sp.wrapping_sub(2);
    let return_address = GAME_ENTRY_POINT + 3;
    let [return_low, return_high] = return_address.to_le_bytes();

    assert_eq!(cpu.registers.get_pc(), target);
    assert_eq!(cpu.registers.get_sp(), expected_sp);
    assert_eq!(cpu.bus.read(expected_sp), return_low);
    assert_eq!(cpu.bus.read(expected_sp + 1), return_high);
}

#[test]
fn call_conditions() {
    let cases = [
        (0xC4, Condition::NotZero),
        (0xCC, Condition::Zero),
        (0xD4, Condition::NotCarry),
        (0xDC, Condition::Carry),
    ];

    let mut rng = rand::rng();

    for (opcode, condition) in cases {
        let target = rng.random_range(u16::MIN..=u16::MAX);
        let [low, high] = target.to_le_bytes();

        let mut cpu = create_cpu(opcode, Some(low), Some(high));

        let sp = get_rand_wram_address();
        cpu.registers.set_sp(sp);

        match condition {
            Condition::NotZero => cpu.registers.set_zero(false),
            Condition::Zero => cpu.registers.set_zero(true),
            Condition::NotCarry => cpu.registers.set_carry(false),
            Condition::Carry => cpu.registers.set_carry(true),
        }

        cpu.step();

        let expected_sp = sp.wrapping_sub(2);
        let return_address = GAME_ENTRY_POINT + 3;
        let [return_low, return_high] = return_address.to_le_bytes();

        assert_eq!(cpu.registers.get_pc(), target);
        assert_eq!(cpu.registers.get_sp(), expected_sp);
        assert_eq!(cpu.bus.read(expected_sp), return_low);
        assert_eq!(cpu.bus.read(expected_sp + 1), return_high);
    }
}

#[test]
fn call_condition_not_met() {
    let cases = [
        (0xC4, Condition::NotZero),
        (0xCC, Condition::Zero),
        (0xD4, Condition::NotCarry),
        (0xDC, Condition::Carry),
    ];

    let mut rng = rand::rng();

    for (opcode, condition) in cases {
        let target = rng.random_range(u16::MIN..=u16::MAX);
        let [low, high] = target.to_le_bytes();

        let mut cpu = create_cpu(opcode, Some(low), Some(high));

        let sp = get_rand_wram_address();
        cpu.registers.set_sp(sp);

        match condition {
            Condition::NotZero => cpu.registers.set_zero(true),
            Condition::Zero => cpu.registers.set_zero(false),
            Condition::NotCarry => cpu.registers.set_carry(true),
            Condition::Carry => cpu.registers.set_carry(false),
        }

        cpu.step();

        assert_eq!(cpu.registers.get_pc(), GAME_ENTRY_POINT + 3);
        assert_eq!(cpu.registers.get_sp(), sp);
    }
}
