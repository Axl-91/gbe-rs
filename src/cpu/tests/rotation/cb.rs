use super::*;

#[test]
fn rlc_register() {
    let mut rng = rand::rng();
    let value = rng.random_range(1..=u8::MAX);

    let mut cpu = create_cpu(0xCB, Some(0x00), None);
    cpu.registers.set_b(value);

    cpu.step();

    let expected_carry = value & 0x80 != 0;
    let expected_value = value.rotate_left(1);

    assert_eq!(cpu.registers.get_b(), expected_value);
    assert_eq!(cpu.registers.get_zero(), expected_value == 0);
    assert!(!cpu.registers.get_subtract());
    assert!(!cpu.registers.get_half_carry());
    assert_eq!(cpu.registers.get_carry(), expected_carry);
}

#[test]
fn rlc_from_hl() {
    let mut rng = rand::rng();
    let value = rng.random_range(1..=u8::MAX);
    let address = get_rand_wram_address();

    let mut cpu = create_cpu(0xCB, Some(0x06), None);
    cpu.registers.set_hl(address);
    cpu.bus.write(address, value);

    cpu.step();

    let expected_carry = value & 0x80 != 0;
    let expected_value = value.rotate_left(1);

    assert_eq!(cpu.bus.read(address), expected_value);
    assert_eq!(cpu.registers.get_zero(), expected_value == 0);
    assert!(!cpu.registers.get_subtract());
    assert!(!cpu.registers.get_half_carry());
    assert_eq!(cpu.registers.get_carry(), expected_carry);
}

#[test]
fn rrc_register() {
    let mut rng = rand::rng();
    let value = rng.random_range(1..=u8::MAX);

    let mut cpu = create_cpu(0xCB, Some(0x08), None);
    cpu.registers.set_b(value);

    cpu.step();

    let expected_carry = value & 0x01 != 0;
    let expected_value = value.rotate_right(1);

    assert_eq!(cpu.registers.get_b(), expected_value);
    assert_eq!(cpu.registers.get_zero(), expected_value == 0);
    assert!(!cpu.registers.get_subtract());
    assert!(!cpu.registers.get_half_carry());
    assert_eq!(cpu.registers.get_carry(), expected_carry);
}

#[test]
fn rrc_from_hl() {
    let mut rng = rand::rng();
    let value = rng.random_range(1..=u8::MAX);
    let address = get_rand_wram_address();

    let mut cpu = create_cpu(0xCB, Some(0x0E), None);
    cpu.registers.set_hl(address);
    cpu.bus.write(address, value);

    cpu.step();

    let expected_carry = value & 0x01 != 0;
    let expected_value = value.rotate_right(1);

    assert_eq!(cpu.bus.read(address), expected_value);
    assert_eq!(cpu.registers.get_zero(), expected_value == 0);
    assert!(!cpu.registers.get_subtract());
    assert!(!cpu.registers.get_half_carry());
    assert_eq!(cpu.registers.get_carry(), expected_carry);
}

#[test]
fn rl_register() {
    let mut rng = rand::rng();
    let value = rng.random_range(1..=u8::MAX);
    let carry = rng.random_bool(0.5);

    let mut cpu = create_cpu(0xCB, Some(0x10), None);
    cpu.registers.set_b(value);
    cpu.registers.set_carry(carry);

    cpu.step();

    let expected_carry = value & 0x80 != 0;
    let expected_value = (value << 1) | carry as u8;

    assert_eq!(cpu.registers.get_b(), expected_value);
    assert_eq!(cpu.registers.get_zero(), expected_value == 0);
    assert!(!cpu.registers.get_subtract());
    assert!(!cpu.registers.get_half_carry());
    assert_eq!(cpu.registers.get_carry(), expected_carry);
}

#[test]
fn rl_from_hl() {
    let mut rng = rand::rng();
    let value = rng.random_range(1..=u8::MAX);
    let carry = rng.random_bool(0.5);
    let address = get_rand_wram_address();

    let mut cpu = create_cpu(0xCB, Some(0x16), None);
    cpu.registers.set_hl(address);
    cpu.registers.set_carry(carry);
    cpu.bus.write(address, value);

    cpu.step();

    let expected_carry = value & 0x80 != 0;
    let expected_value = (value << 1) | carry as u8;

    assert_eq!(cpu.bus.read(address), expected_value);
    assert_eq!(cpu.registers.get_zero(), expected_value == 0);
    assert!(!cpu.registers.get_subtract());
    assert!(!cpu.registers.get_half_carry());
    assert_eq!(cpu.registers.get_carry(), expected_carry);
}

#[test]
fn rr_register() {
    let mut rng = rand::rng();
    let value = rng.random_range(1..=u8::MAX);
    let carry = rng.random_bool(0.5);

    let mut cpu = create_cpu(0xCB, Some(0x18), None);
    cpu.registers.set_b(value);
    cpu.registers.set_carry(carry);

    cpu.step();

    let expected_carry = value & 0x01 != 0;
    let expected_value = (value >> 1) | ((carry as u8) << 7);

    assert_eq!(cpu.registers.get_b(), expected_value);
    assert_eq!(cpu.registers.get_zero(), expected_value == 0);
    assert!(!cpu.registers.get_subtract());
    assert!(!cpu.registers.get_half_carry());
    assert_eq!(cpu.registers.get_carry(), expected_carry);
}

#[test]
fn rr_from_hl() {
    let mut rng = rand::rng();
    let value = rng.random_range(1..=u8::MAX);
    let carry = rng.random_bool(0.5);
    let address = get_rand_wram_address();

    let mut cpu = create_cpu(0xCB, Some(0x1E), None);
    cpu.registers.set_hl(address);
    cpu.registers.set_carry(carry);
    cpu.bus.write(address, value);

    cpu.step();

    let expected_carry = value & 0x01 != 0;
    let expected_value = (value >> 1) | ((carry as u8) << 7);

    assert_eq!(cpu.bus.read(address), expected_value);
    assert_eq!(cpu.registers.get_zero(), expected_value == 0);
    assert!(!cpu.registers.get_subtract());
    assert!(!cpu.registers.get_half_carry());
    assert_eq!(cpu.registers.get_carry(), expected_carry);
}

#[test]
fn sla_register() {
    let mut rng = rand::rng();
    let value = rng.random_range(1..=u8::MAX);

    let mut cpu = create_cpu(0xCB, Some(0x20), None);
    cpu.registers.set_b(value);

    cpu.step();

    let expected_carry = value & 0x80 != 0;
    let expected_value = value << 1;

    assert_eq!(cpu.registers.get_b(), expected_value);
    assert_eq!(cpu.registers.get_zero(), expected_value == 0);
    assert!(!cpu.registers.get_subtract());
    assert!(!cpu.registers.get_half_carry());
    assert_eq!(cpu.registers.get_carry(), expected_carry);
}

#[test]
fn sla_from_hl() {
    let mut rng = rand::rng();
    let value = rng.random_range(1..=u8::MAX);
    let address = get_rand_wram_address();

    let mut cpu = create_cpu(0xCB, Some(0x26), None);
    cpu.registers.set_hl(address);
    cpu.bus.write(address, value);

    cpu.step();

    let expected_carry = value & 0x80 != 0;
    let expected_value = value << 1;

    assert_eq!(cpu.bus.read(address), expected_value);
    assert_eq!(cpu.registers.get_zero(), expected_value == 0);
    assert!(!cpu.registers.get_subtract());
    assert!(!cpu.registers.get_half_carry());
    assert_eq!(cpu.registers.get_carry(), expected_carry);
}

#[test]
fn sra_register() {
    let mut rng = rand::rng();
    let value = rng.random_range(1..=u8::MAX);

    let mut cpu = create_cpu(0xCB, Some(0x28), None);
    cpu.registers.set_b(value);

    cpu.step();

    let expected_carry = value & 0x01 != 0;
    let expected_value = (value >> 1) | (value & 0x80);

    assert_eq!(cpu.registers.get_b(), expected_value);
    assert_eq!(cpu.registers.get_zero(), expected_value == 0);
    assert!(!cpu.registers.get_subtract());
    assert!(!cpu.registers.get_half_carry());
    assert_eq!(cpu.registers.get_carry(), expected_carry);
}

#[test]
fn sra_from_hl() {
    let mut rng = rand::rng();
    let value = rng.random_range(1..=u8::MAX);
    let address = get_rand_wram_address();

    let mut cpu = create_cpu(0xCB, Some(0x2E), None);
    cpu.registers.set_hl(address);
    cpu.bus.write(address, value);

    cpu.step();

    let expected_carry = value & 0x01 != 0;
    let expected_value = (value >> 1) | (value & 0x80);

    assert_eq!(cpu.bus.read(address), expected_value);
    assert_eq!(cpu.registers.get_zero(), expected_value == 0);
    assert!(!cpu.registers.get_subtract());
    assert!(!cpu.registers.get_half_carry());
    assert_eq!(cpu.registers.get_carry(), expected_carry);
}

#[test]
fn swap_register() {
    let mut rng = rand::rng();
    let value = rng.random_range(1..=u8::MAX);

    let mut cpu = create_cpu(0xCB, Some(0x30), None);
    cpu.registers.set_b(value);

    cpu.step();

    let expected_value = value.rotate_left(4);

    assert_eq!(cpu.registers.get_b(), expected_value);
    assert_eq!(cpu.registers.get_zero(), expected_value == 0);
    assert!(!cpu.registers.get_subtract());
    assert!(!cpu.registers.get_half_carry());
    assert!(!cpu.registers.get_carry());
}

#[test]
fn swap_from_hl() {
    let mut rng = rand::rng();
    let value = rng.random_range(1..=u8::MAX);
    let address = get_rand_wram_address();

    let mut cpu = create_cpu(0xCB, Some(0x36), None);
    cpu.registers.set_hl(address);
    cpu.bus.write(address, value);

    cpu.step();

    let expected_value = value.rotate_left(4);

    assert_eq!(cpu.bus.read(address), expected_value);
    assert_eq!(cpu.registers.get_zero(), expected_value == 0);
    assert!(!cpu.registers.get_subtract());
    assert!(!cpu.registers.get_half_carry());
    assert!(!cpu.registers.get_carry());
}

#[test]
fn srl_register() {
    let mut rng = rand::rng();
    let value = rng.random_range(1..=u8::MAX);

    let mut cpu = create_cpu(0xCB, Some(0x38), None);
    cpu.registers.set_b(value);

    cpu.step();

    let expected_carry = value & 0x01 != 0;
    let expected_value = value >> 1;

    assert_eq!(cpu.registers.get_b(), expected_value);
    assert_eq!(cpu.registers.get_zero(), expected_value == 0);
    assert!(!cpu.registers.get_subtract());
    assert!(!cpu.registers.get_half_carry());
    assert_eq!(cpu.registers.get_carry(), expected_carry);
}

#[test]
fn srl_from_hl() {
    let mut rng = rand::rng();
    let value = rng.random_range(1..=u8::MAX);
    let address = get_rand_wram_address();

    let mut cpu = create_cpu(0xCB, Some(0x3E), None);
    cpu.registers.set_hl(address);
    cpu.bus.write(address, value);

    cpu.step();

    let expected_carry = value & 0x01 != 0;
    let expected_value = value >> 1;

    assert_eq!(cpu.bus.read(address), expected_value);
    assert_eq!(cpu.registers.get_zero(), expected_value == 0);
    assert!(!cpu.registers.get_subtract());
    assert!(!cpu.registers.get_half_carry());
    assert_eq!(cpu.registers.get_carry(), expected_carry);
}
