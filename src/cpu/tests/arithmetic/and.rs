use super::*;

#[test]
fn and_register() {
    let mut rng = rand::rng();

    let opcodes = [0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5, 0xA7];
    let opcode = opcodes[rng.random_range(0..opcodes.len())];

    let a = rng.random_range(0..=u8::MAX);
    let value = rng.random_range(0..=u8::MAX);

    let mut cpu = create_cpu(opcode, None, None);
    cpu.registers.set_a(a);

    match opcode {
        0xA0 => cpu.registers.set_b(value),
        0xA1 => cpu.registers.set_c(value),
        0xA2 => cpu.registers.set_d(value),
        0xA3 => cpu.registers.set_e(value),
        0xA4 => cpu.registers.set_h(value),
        0xA5 => cpu.registers.set_l(value),
        0xA7 => {}
        _ => unreachable!(),
    }

    let expected = match opcode {
        0xA7 => a & a,
        _ => a & value,
    };

    cpu.step();

    assert_eq!(cpu.registers.get_a(), expected);
    assert_eq!(cpu.registers.get_zero(), expected == 0);
    assert!(!cpu.registers.get_subtract());
    assert!(cpu.registers.get_half_carry());
    assert!(!cpu.registers.get_carry());
}

#[test]
fn and_from_hl() {
    let mut rng = rand::rng();

    let a = rng.random_range(0..=u8::MAX);
    let value = rng.random_range(0..=u8::MAX);
    let address = get_rand_wram_address();

    let mut cpu = create_cpu(0xA6, None, None);

    cpu.registers.set_a(a);
    cpu.registers.set_hl(address);
    cpu.bus.write(address, value);

    let expected = a & value;

    cpu.step();

    assert_eq!(cpu.registers.get_a(), expected);
    assert_eq!(cpu.registers.get_zero(), expected == 0);
    assert!(!cpu.registers.get_subtract());
    assert!(cpu.registers.get_half_carry());
    assert!(!cpu.registers.get_carry());
}

#[test]
fn and_immediate() {
    let mut rng = rand::rng();

    let a = rng.random_range(0..=u8::MAX);
    let value = rng.random_range(0..=u8::MAX);

    let mut cpu = create_cpu(0xE6, Some(value), None);

    cpu.registers.set_a(a);

    let expected = a & value;

    cpu.step();

    assert_eq!(cpu.registers.get_a(), expected);
    assert_eq!(cpu.registers.get_zero(), expected == 0);
    assert!(!cpu.registers.get_subtract());
    assert!(cpu.registers.get_half_carry());
    assert!(!cpu.registers.get_carry());
    assert_eq!(cpu.registers.get_pc(), GAME_ENTRY_POINT + 2);
}
