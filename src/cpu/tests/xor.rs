use super::*;

#[test]
fn xor_register() {
    let mut rng = rand::rng();

    let opcodes = [0xA8, 0xA9, 0xAA, 0xAB, 0xAC, 0xAD, 0xAF];

    for opcode in opcodes {
        let a = rng.random_range(0..=u8::MAX);
        let value = rng.random_range(0..=u8::MAX);

        let mut cpu = create_cpu(opcode, None, None);
        cpu.registers.set_a(a);

        match opcode {
            0xA8 => cpu.registers.set_b(value),
            0xA9 => cpu.registers.set_c(value),
            0xAA => cpu.registers.set_d(value),
            0xAB => cpu.registers.set_e(value),
            0xAC => cpu.registers.set_h(value),
            0xAD => cpu.registers.set_l(value),
            0xAF => {}
            _ => unreachable!(),
        }

        let expected = match opcode {
            0xAF => a ^ a,
            _ => a ^ value,
        };

        cpu.step();

        assert_eq!(cpu.registers.get_a(), expected);
        assert_eq!(cpu.registers.get_zero(), expected == 0);
        assert!(!cpu.registers.get_subtract());
        assert!(!cpu.registers.get_half_carry());
        assert!(!cpu.registers.get_carry());
    }
}

#[test]
fn xor_from_hl() {
    let mut rng = rand::rng();

    let a = rng.random_range(0..=u8::MAX);
    let value = rng.random_range(0..=u8::MAX);
    let address = get_rand_wram_address();

    let mut cpu = create_cpu(0xAE, None, None);

    cpu.registers.set_a(a);
    cpu.registers.set_hl(address);
    cpu.bus.write(address, value);

    let expected = a ^ value;

    cpu.step();

    assert_eq!(cpu.registers.get_a(), expected);
    assert_eq!(cpu.registers.get_zero(), expected == 0);
    assert!(!cpu.registers.get_subtract());
    assert!(!cpu.registers.get_half_carry());
    assert!(!cpu.registers.get_carry());
}

#[test]
fn xor_immediate() {
    let mut rng = rand::rng();

    let a = rng.random_range(0..=u8::MAX);
    let value = rng.random_range(0..=u8::MAX);

    let mut cpu = create_cpu(0xEE, Some(value), None);

    cpu.registers.set_a(a);

    let expected = a ^ value;

    cpu.step();

    assert_eq!(cpu.registers.get_a(), expected);
    assert_eq!(cpu.registers.get_zero(), expected == 0);
    assert!(!cpu.registers.get_subtract());
    assert!(!cpu.registers.get_half_carry());
    assert!(!cpu.registers.get_carry());
    assert_eq!(cpu.registers.get_pc(), GAME_ENTRY_POINT + 2);
}
