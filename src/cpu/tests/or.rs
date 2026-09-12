use super::*;

#[test]
fn or_register() {
    let mut rng = rand::rng();

    let opcodes = [0xB0, 0xB1, 0xB2, 0xB3, 0xB4, 0xB5, 0xB7];
    let opcode = opcodes[rng.random_range(0..opcodes.len())];

    let a = rng.random_range(0..=u8::MAX);
    let value = rng.random_range(0..=u8::MAX);

    let mut cpu = create_cpu(opcode, None, None);
    cpu.registers.set_a(a);

    match opcode {
        0xB0 => cpu.registers.set_b(value),
        0xB1 => cpu.registers.set_c(value),
        0xB2 => cpu.registers.set_d(value),
        0xB3 => cpu.registers.set_e(value),
        0xB4 => cpu.registers.set_h(value),
        0xB5 => cpu.registers.set_l(value),
        0xB7 => {}
        _ => unreachable!(),
    }

    let expected = match opcode {
        0xB7 => a | a,
        _ => a | value,
    };

    cpu.step();

    assert_eq!(cpu.registers.get_a(), expected);
    assert_eq!(cpu.registers.get_zero(), expected == 0);
    assert!(!cpu.registers.get_subtract());
    assert!(!cpu.registers.get_half_carry());
    assert!(!cpu.registers.get_carry());
}

#[test]
fn or_from_hl() {
    let mut rng = rand::rng();

    let a = rng.random_range(0..=u8::MAX);
    let value = rng.random_range(0..=u8::MAX);
    let address = get_rand_wram_address();

    let mut cpu = create_cpu(0xB6, None, None);

    cpu.registers.set_a(a);
    cpu.registers.set_hl(address);
    cpu.bus.write(address, value);

    let expected = a | value;

    cpu.step();

    assert_eq!(cpu.registers.get_a(), expected);
    assert_eq!(cpu.registers.get_zero(), expected == 0);
    assert!(!cpu.registers.get_subtract());
    assert!(!cpu.registers.get_half_carry());
    assert!(!cpu.registers.get_carry());
}

#[test]
fn or_immediate() {
    let mut rng = rand::rng();

    let a = rng.random_range(0..=u8::MAX);
    let value = rng.random_range(0..=u8::MAX);

    let mut cpu = create_cpu(0xF6, Some(value), None);

    cpu.registers.set_a(a);

    let expected = a | value;

    cpu.step();

    assert_eq!(cpu.registers.get_a(), expected);
    assert_eq!(cpu.registers.get_zero(), expected == 0);
    assert!(!cpu.registers.get_subtract());
    assert!(!cpu.registers.get_half_carry());
    assert!(!cpu.registers.get_carry());
    assert_eq!(cpu.registers.get_pc(), GAME_ENTRY_POINT + 2);
}
