use super::*;

#[test]
fn sub_register() {
    let mut rng = rand::rng();

    let registers = [
        (0x90, Register8::B),
        (0x91, Register8::C),
        (0x92, Register8::D),
        (0x93, Register8::E),
        (0x94, Register8::H),
        (0x95, Register8::L),
    ];

    let index = rng.random_range(0..registers.len());
    let (opcode, register) = registers[index];

    let a = rng.random::<u8>();
    let value = rng.random::<u8>();

    let mut cpu = create_cpu(opcode, None, None);

    cpu.registers.set_a(a);
    cpu.set_register8(&register, value);

    cpu.step();

    assert_eq!(cpu.registers.get_a(), a.wrapping_sub(value));
}

#[test]
fn sub_register_a() {
    let mut rng = rand::rng();

    let a = rng.random::<u8>();

    let mut cpu = create_cpu(0x97, None, None);

    cpu.registers.set_a(a);

    cpu.step();

    assert_eq!(cpu.registers.get_a(), 0);
    assert!(cpu.registers.get_zero());
    assert!(cpu.registers.get_subtract());
}

#[test]
fn sub_from_hl() {
    let mut rng = rand::rng();

    let a = rng.random::<u8>();
    let value = rng.random::<u8>();
    let address = get_rand_wram_address();

    let mut cpu = create_cpu(0x96, None, None);

    cpu.registers.set_a(a);
    cpu.registers.set_hl(address);
    cpu.bus.write(address, value);

    cpu.step();

    assert_eq!(cpu.registers.get_a(), a.wrapping_sub(value));
}

#[test]
fn sub_immediate() {
    let mut rng = rand::rng();

    let a = rng.random::<u8>();
    let value = rng.random::<u8>();

    let mut cpu = create_cpu(0xD6, Some(value), None);

    cpu.registers.set_a(a);

    cpu.step();

    assert_eq!(cpu.registers.get_a(), a.wrapping_sub(value));
}

#[test]
fn sub_sets_zero_flag() {
    let mut rng = rand::rng();

    let value = rng.random::<u8>();

    let mut cpu = create_cpu(0x97, None, None);

    cpu.registers.set_a(value);

    cpu.step();

    assert!(cpu.registers.get_zero());
}

#[test]
fn sub_clears_zero_flag() {
    let mut rng = rand::rng();

    let value = rng.random::<u8>();
    let a = value.wrapping_add(1);

    let mut cpu = create_cpu(0xD6, Some(value), None);

    cpu.registers.set_a(a);

    cpu.step();

    assert!(!cpu.registers.get_zero());
}

#[test]
fn sub_sets_subtract_flag() {
    let mut rng = rand::rng();

    let a = rng.random::<u8>();
    let value = rng.random::<u8>();

    let mut cpu = create_cpu(0xD6, Some(value), None);

    cpu.registers.set_a(a);

    cpu.step();

    assert!(cpu.registers.get_subtract());
}

#[test]
fn sub_sets_half_carry() {
    let mut cpu = create_cpu(0xD6, Some(1), None);

    cpu.registers.set_a(0);

    cpu.step();

    assert!(cpu.registers.get_half_carry());
}

#[test]
fn sub_clears_half_carry() {
    let mut rng = rand::rng();

    let value = rng.random_range(0..=0x0F);
    let low_nibble = rng.random_range(value..=0x0F);
    let high_nibble = rng.random::<u8>() & 0xF0;
    let a = high_nibble | low_nibble;

    let mut cpu = create_cpu(0xD6, Some(value), None);

    cpu.registers.set_a(a);

    cpu.step();

    assert!(!cpu.registers.get_half_carry());
}

#[test]
fn sub_sets_carry() {
    let mut rng = rand::rng();

    let value = rng.random_range(1..=u8::MAX);

    let mut cpu = create_cpu(0xD6, Some(value), None);

    cpu.registers.set_a(0);

    cpu.step();

    assert!(cpu.registers.get_carry());
}

#[test]
fn sub_clears_carry() {
    let mut rng = rand::rng();

    let value = rng.random::<u8>();
    let a = rng.random_range(value..=u8::MAX);

    let mut cpu = create_cpu(0xD6, Some(value), None);

    cpu.registers.set_a(a);

    cpu.step();

    assert!(!cpu.registers.get_carry());
}

#[test]
fn sub_register_advances_pc_by_one() {
    let mut rng = rand::rng();

    let registers = [
        (0x90, Register8::B),
        (0x91, Register8::C),
        (0x92, Register8::D),
        (0x93, Register8::E),
        (0x94, Register8::H),
        (0x95, Register8::L),
        (0x97, Register8::A),
    ];

    let index = rng.random_range(0..registers.len());
    let (opcode, register) = registers[index];

    let mut cpu = create_cpu(opcode, None, None);

    cpu.registers.set_a(rng.random());
    cpu.set_register8(&register, rng.random());

    let initial_pc = cpu.registers.get_pc();

    cpu.step();

    assert_eq!(cpu.registers.get_pc(), initial_pc + 1);
}

#[test]
fn sub_from_hl_advances_pc_by_one() {
    let address = get_rand_wram_address();

    let mut cpu = create_cpu(0x96, None, None);

    cpu.registers.set_a(1);
    cpu.registers.set_hl(address);
    cpu.bus.write(address, 1);

    let initial_pc = cpu.registers.get_pc();

    cpu.step();

    assert_eq!(cpu.registers.get_pc(), initial_pc + 1);
}

#[test]
fn sub_immediate_advances_pc_by_two() {
    let mut rng = rand::rng();

    let mut cpu = create_cpu(0xD6, Some(rng.random()), None);

    cpu.registers.set_a(rng.random());

    let initial_pc = cpu.registers.get_pc();

    cpu.step();

    assert_eq!(cpu.registers.get_pc(), initial_pc + 2);
}
