use super::*;

#[test]
fn bit_register() {
    let mut rng = rand::rng();

    let value = rng.random_range(0..=u8::MAX);
    let bit = rng.random_range(0..=7);

    let opcode = 0x40 | (bit << 3);

    let mut cpu = create_cpu(0xCB, Some(opcode), None);
    cpu.registers.set_b(value);

    cpu.step();

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

    let opcode = 0x40 | (bit << 3) | 0x06;

    let mut cpu = create_cpu(0xCB, Some(opcode), None);
    cpu.registers.set_hl(address);
    cpu.bus.write(address, value);

    cpu.step();

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

    let opcode = 0x80 | (bit << 3);

    let mut cpu = create_cpu(0xCB, Some(opcode), None);
    cpu.registers.set_b(value);

    cpu.step();

    let expected = value & !(1 << bit);

    assert_eq!(cpu.registers.get_b(), expected);
}

#[test]
fn res_from_hl() {
    let mut rng = rand::rng();

    let value = rng.random_range(0..=u8::MAX);
    let bit = rng.random_range(0..=7);
    let address = get_rand_wram_address();

    let opcode = 0x80 | (bit << 3) | 0x06;

    let mut cpu = create_cpu(0xCB, Some(opcode), None);
    cpu.registers.set_hl(address);
    cpu.bus.write(address, value);

    cpu.step();

    let expected = value & !(1 << bit);

    assert_eq!(cpu.bus.read(address), expected);
}

#[test]
fn set_register() {
    let mut rng = rand::rng();

    let value = rng.random_range(0..=u8::MAX);
    let bit = rng.random_range(0..=7);

    let opcode = 0xC0 | (bit << 3);

    let mut cpu = create_cpu(0xCB, Some(opcode), None);
    cpu.registers.set_b(value);

    cpu.step();

    let expected = value | (1 << bit);

    assert_eq!(cpu.registers.get_b(), expected);
}

#[test]
fn set_from_hl() {
    let mut rng = rand::rng();

    let value = rng.random_range(0..=u8::MAX);
    let bit = rng.random_range(0..=7);
    let address = get_rand_wram_address();

    let opcode = 0xC0 | (bit << 3) | 0x06;

    let mut cpu = create_cpu(0xCB, Some(opcode), None);
    cpu.registers.set_hl(address);
    cpu.bus.write(address, value);

    cpu.step();

    let expected = value | (1 << bit);

    assert_eq!(cpu.bus.read(address), expected);
}
