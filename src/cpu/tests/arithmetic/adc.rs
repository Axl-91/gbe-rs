use super::*;

#[test]
fn adc_register() {
    let mut rng = rand::rng();

    let opcodes = [0x8F];
    let opcode = opcodes[rng.random_range(0..opcodes.len())];

    let mut value = rng.random();
    let a = rng.random();

    let mut cpu = create_cpu(opcode, None, None);

    cpu.registers.set_a(a);
    cpu.registers.set_carry(false);

    let register = match opcode {
        0x88 => Register8::B,
        0x89 => Register8::C,
        0x8A => Register8::D,
        0x8B => Register8::E,
        0x8C => Register8::H,
        0x8D => Register8::L,
        0x8F => Register8::A,
        _ => panic!("Invalid ADC register opcode"),
    };

    if opcode == 0x08F {
        value = a
    } else {
        cpu.set_register8(&register, value);
    }

    cpu.step();

    assert_eq!(cpu.registers.get_a(), a.wrapping_add(value));
}

#[test]
fn adc_register_with_carry() {
    let mut rng = rand::rng();

    let opcodes = [0x88, 0x89, 0x8A, 0x8B, 0x8C, 0x8D, 0x8F];
    let opcode = opcodes[rng.random_range(0..opcodes.len())];

    let mut value = rng.random();
    let a = rng.random();

    let mut cpu = create_cpu(opcode, None, None);

    cpu.registers.set_a(a);
    cpu.registers.set_carry(true);

    let register = match opcode {
        0x88 => Register8::B,
        0x89 => Register8::C,
        0x8A => Register8::D,
        0x8B => Register8::E,
        0x8C => Register8::H,
        0x8D => Register8::L,
        0x8F => Register8::A,
        _ => panic!("Invalid ADC register opcode"),
    };

    if opcode == 0x08F {
        value = a
    } else {
        cpu.set_register8(&register, value);
    }

    cpu.step();

    assert_eq!(cpu.registers.get_a(), a.wrapping_add(value).wrapping_add(1));
}

#[test]
fn adc_from_hl() {
    let mut rng = rand::rng();

    let mut cpu = create_cpu(0x8E, None, None);

    let address = get_rand_wram_address();
    let a = rng.random();
    let value = rng.random();

    cpu.registers.set_hl(address);
    cpu.registers.set_a(a);
    cpu.registers.set_carry(false);
    cpu.bus.write(address, value);

    cpu.step();

    assert_eq!(cpu.registers.get_a(), a.wrapping_add(value));
}

#[test]
fn adc_from_hl_with_carry() {
    let mut rng = rand::rng();

    let mut cpu = create_cpu(0x8E, None, None);

    let address = get_rand_wram_address();
    let a = rng.random();
    let value = rng.random();

    cpu.registers.set_hl(address);
    cpu.registers.set_a(a);
    cpu.registers.set_carry(true);
    cpu.bus.write(address, value);

    cpu.step();

    assert_eq!(cpu.registers.get_a(), a.wrapping_add(value).wrapping_add(1));
}

#[test]
fn adc_immediate() {
    let mut rng = rand::rng();

    let value = rng.random();
    let a = rng.random();

    let mut cpu = create_cpu(0xCE, Some(value), None);

    cpu.registers.set_a(a);
    cpu.registers.set_carry(false);

    cpu.step();

    assert_eq!(cpu.registers.get_a(), a.wrapping_add(value));
}

#[test]
fn adc_immediate_with_carry() {
    let mut rng = rand::rng();

    let value = rng.random();
    let a = rng.random();

    let mut cpu = create_cpu(0xCE, Some(value), None);

    cpu.registers.set_a(a);
    cpu.registers.set_carry(true);

    cpu.step();

    assert_eq!(cpu.registers.get_a(), a.wrapping_add(value).wrapping_add(1));
}

#[test]
fn adc_sets_zero_flag() {
    let mut cpu = create_cpu(0xCE, Some(0), None);

    cpu.registers.set_a(0);
    cpu.registers.set_carry(false);

    cpu.step();

    assert!(cpu.registers.get_zero());
}

#[test]
fn adc_clears_zero_flag() {
    let mut cpu = create_cpu(0xCE, Some(1), None);

    cpu.registers.set_a(1);
    cpu.registers.set_carry(false);

    cpu.step();

    assert!(!cpu.registers.get_zero());
}

#[test]
fn adc_clears_subtract_flag() {
    let mut cpu = create_cpu(0xCE, Some(1), None);

    cpu.registers.set_a(1);
    cpu.registers.set_subtract(true);

    cpu.step();

    assert!(!cpu.registers.get_subtract());
}

#[test]
fn adc_sets_half_carry() {
    let mut cpu = create_cpu(0xCE, Some(1), None);

    cpu.registers.set_a(0x0F);
    cpu.registers.set_carry(false);

    cpu.step();

    assert!(cpu.registers.get_half_carry());
}

#[test]
fn adc_sets_half_carry_with_carry() {
    let mut cpu = create_cpu(0xCE, Some(0), None);

    cpu.registers.set_a(0x0F);
    cpu.registers.set_carry(true);

    cpu.step();

    assert!(cpu.registers.get_half_carry());
}

#[test]
fn adc_sets_carry() {
    let mut cpu = create_cpu(0xCE, Some(1), None);

    cpu.registers.set_a(u8::MAX);
    cpu.registers.set_carry(false);

    cpu.step();

    assert!(cpu.registers.get_carry());
}

#[test]
fn adc_sets_carry_with_carry() {
    let mut cpu = create_cpu(0xCE, Some(u8::MAX), None);

    cpu.registers.set_a(u8::MAX);
    cpu.registers.set_carry(true);

    cpu.step();

    assert!(cpu.registers.get_carry());
}

#[test]
fn adc_register_advances_pc_by_one() {
    let mut cpu = create_cpu(0x88, None, None);

    cpu.step();

    assert_eq!(cpu.registers.get_pc(), GAME_ENTRY_POINT + 1);
}

#[test]
fn adc_from_hl_advances_pc_by_one() {
    let mut cpu = create_cpu(0x8E, None, None);

    cpu.step();

    assert_eq!(cpu.registers.get_pc(), GAME_ENTRY_POINT + 1);
}

#[test]
fn adc_immediate_advances_pc_by_two() {
    let mut cpu = create_cpu(0xCE, Some(1), None);

    cpu.step();

    assert_eq!(cpu.registers.get_pc(), GAME_ENTRY_POINT + 2);
}
