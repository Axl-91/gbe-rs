use super::*;

#[test]
fn bit_register() {
    let mut rng = rand::rng();

    let value = rng.random_range(0..=u8::MAX);
    let bit = rng.random_range(0..=7);

    let mut cpu = create_cpu(0, None, None);
    cpu.registers.set_b(value);

    cpu.execute_cb(CbInstruction::Bit(bit, Register8::B));

    let expected_zero = value & (1 << bit) == 0;

    assert_eq!(cpu.registers.get_zero(), expected_zero);
    assert!(!cpu.registers.get_subtract());
    assert!(cpu.registers.get_half_carry());
}

#[test]
fn bit_from_hl() {
    let mut rng = rand::rng();

    let value = rng.random_range(0..=u8::MAX);
    let bit = rng.random_range(0..=7);
    let address = get_rand_wram_address();

    let mut cpu = create_cpu(0, None, None);
    cpu.registers.set_hl(address);
    cpu.bus.write(address, value);

    cpu.execute_cb(CbInstruction::BitFromHl(bit));

    let expected_zero = value & (1 << bit) == 0;

    assert_eq!(cpu.registers.get_zero(), expected_zero);
    assert!(!cpu.registers.get_subtract());
    assert!(cpu.registers.get_half_carry());
}

#[test]
fn res_register() {
    let mut rng = rand::rng();

    let value = rng.random_range(0..=u8::MAX);
    let bit = rng.random_range(0..=7);

    let mut cpu = create_cpu(0, None, None);
    cpu.registers.set_b(value);

    cpu.execute_cb(CbInstruction::Res(bit, Register8::B));

    let expected = value & !(1 << bit);

    assert_eq!(cpu.registers.get_b(), expected);
}

#[test]
fn res_from_hl() {
    let mut rng = rand::rng();

    let value = rng.random_range(0..=u8::MAX);
    let bit = rng.random_range(0..=7);
    let address = get_rand_wram_address();

    let mut cpu = create_cpu(0, None, None);
    cpu.registers.set_hl(address);
    cpu.bus.write(address, value);

    cpu.execute_cb(CbInstruction::ResFromHl(bit));

    let expected = value & !(1 << bit);

    assert_eq!(cpu.bus.read(address), expected);
}

#[test]
fn set_register() {
    let mut rng = rand::rng();

    let value = rng.random_range(0..=u8::MAX);
    let bit = rng.random_range(0..=7);

    let mut cpu = create_cpu(0, None, None);
    cpu.registers.set_b(value);

    cpu.execute_cb(CbInstruction::Set(bit, Register8::B));

    let expected = value | (1 << bit);

    assert_eq!(cpu.registers.get_b(), expected);
}

#[test]
fn set_from_hl() {
    let mut rng = rand::rng();

    let value = rng.random_range(0..=u8::MAX);
    let bit = rng.random_range(0..=7);
    let address = get_rand_wram_address();

    let mut cpu = create_cpu(0, None, None);
    cpu.registers.set_hl(address);
    cpu.bus.write(address, value);

    cpu.execute_cb(CbInstruction::SetFromHl(bit));

    let expected = value | (1 << bit);

    assert_eq!(cpu.bus.read(address), expected);
}
