use super::*;

#[test]
fn cp_register() {
    let mut rng = rand::rng();

    let opcodes = [0xB8, 0xB9, 0xBA, 0xBB, 0xBC, 0xBD, 0xBF];

    for opcode in opcodes {
        let a = rng.random_range(0..=u8::MAX);
        let mut value = rng.random_range(0..=u8::MAX);

        let mut cpu = create_cpu(opcode, None, None);
        cpu.registers.set_a(a);

        match opcode {
            0xB8 => cpu.registers.set_b(value),
            0xB9 => cpu.registers.set_c(value),
            0xBA => cpu.registers.set_d(value),
            0xBB => cpu.registers.set_e(value),
            0xBC => cpu.registers.set_h(value),
            0xBD => cpu.registers.set_l(value),
            0xBF => {}
            _ => unreachable!(),
        }

        if opcode == 0xBF {
            value = a;
        }
        let result = a.wrapping_sub(value);
        let expected_half_carry = (a & 0x0F) < (value & 0x0F);
        let expected_carry = a < value;

        cpu.step();

        assert_eq!(cpu.registers.get_a(), a);
        assert_eq!(cpu.registers.get_zero(), result == 0);
        assert!(cpu.registers.get_subtract());
        assert_eq!(cpu.registers.get_half_carry(), expected_half_carry);
        assert_eq!(cpu.registers.get_carry(), expected_carry);
    }
}

#[test]
fn cp_from_hl() {
    let mut rng = rand::rng();

    let a = rng.random_range(0..=u8::MAX);
    let value = rng.random_range(0..=u8::MAX);
    let address = get_rand_wram_address();

    let mut cpu = create_cpu(0xBE, None, None);

    cpu.registers.set_a(a);
    cpu.registers.set_hl(address);
    cpu.bus.write(address, value);

    let result = a.wrapping_sub(value);
    let expected_half_carry = (a & 0x0F) < (value & 0x0F);
    let expected_carry = a < value;

    cpu.step();

    assert_eq!(cpu.registers.get_a(), a);
    assert_eq!(cpu.registers.get_zero(), result == 0);
    assert!(cpu.registers.get_subtract());
    assert_eq!(cpu.registers.get_half_carry(), expected_half_carry);
    assert_eq!(cpu.registers.get_carry(), expected_carry);
}

#[test]
fn cp_immediate() {
    let mut rng = rand::rng();

    let a = rng.random_range(0..=u8::MAX);
    let value = rng.random_range(0..=u8::MAX);

    let mut cpu = create_cpu(0xFE, Some(value), None);

    cpu.registers.set_a(a);

    let result = a.wrapping_sub(value);
    let expected_half_carry = (a & 0x0F) < (value & 0x0F);
    let expected_carry = a < value;

    cpu.step();

    assert_eq!(cpu.registers.get_a(), a);
    assert_eq!(cpu.registers.get_zero(), result == 0);
    assert!(cpu.registers.get_subtract());
    assert_eq!(cpu.registers.get_half_carry(), expected_half_carry);
    assert_eq!(cpu.registers.get_carry(), expected_carry);
    assert_eq!(cpu.registers.get_pc(), GAME_ENTRY_POINT + 2);
}
