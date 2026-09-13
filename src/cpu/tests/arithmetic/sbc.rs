use super::*;

#[test]
fn sbc_register() {
    let mut rng = rand::rng();

    let registers = [
        (0x98, Register8::B),
        (0x99, Register8::C),
        (0x9A, Register8::D),
        (0x9B, Register8::E),
        (0x9C, Register8::H),
        (0x9D, Register8::L),
    ];

    let index = rng.random_range(0..registers.len());
    let (opcode, register) = registers[index];

    let a = rng.random::<u8>();
    let value = rng.random::<u8>();

    let mut cpu = create_cpu(opcode, None, None);

    cpu.registers.set_a(a);
    cpu.set_register8(&register, value);
    cpu.registers.set_carry(false);

    cpu.step();

    assert_eq!(cpu.registers.get_a(), a.wrapping_sub(value));
}

#[test]
fn sbc_register_with_borrow() {
    let mut rng = rand::rng();

    let registers = [
        (0x98, Register8::B),
        (0x99, Register8::C),
        (0x9A, Register8::D),
        (0x9B, Register8::E),
        (0x9C, Register8::H),
        (0x9D, Register8::L),
    ];

    let index = rng.random_range(0..registers.len());
    let (opcode, register) = registers[index];

    let a = rng.random::<u8>();
    let value = rng.random::<u8>();

    let mut cpu = create_cpu(opcode, None, None);

    cpu.registers.set_a(a);
    cpu.set_register8(&register, value);
    cpu.registers.set_carry(true);

    cpu.step();

    assert_eq!(cpu.registers.get_a(), a.wrapping_sub(value).wrapping_sub(1));
}

#[test]
fn sbc_register_a() {
    let mut rng = rand::rng();

    let a = rng.random::<u8>();

    let mut cpu = create_cpu(0x9F, None, None);

    cpu.registers.set_a(a);
    cpu.registers.set_carry(false);

    cpu.step();

    assert_eq!(cpu.registers.get_a(), 0);
    assert!(cpu.registers.get_zero());
    assert!(cpu.registers.get_subtract());
}

#[test]
fn sbc_from_hl() {
    let mut rng = rand::rng();

    let a = rng.random::<u8>();
    let value = rng.random::<u8>();
    let address = get_rand_wram_address();

    let mut cpu = create_cpu(0x9E, None, None);

    cpu.registers.set_a(a);
    cpu.registers.set_hl(address);
    cpu.registers.set_carry(false);
    cpu.bus.write(address, value);

    cpu.step();

    assert_eq!(cpu.registers.get_a(), a.wrapping_sub(value));
}

#[test]
fn sbc_from_hl_with_borrow() {
    let mut rng = rand::rng();

    let a = rng.random::<u8>();
    let value = rng.random::<u8>();
    let address = get_rand_wram_address();

    let mut cpu = create_cpu(0x9E, None, None);

    cpu.registers.set_a(a);
    cpu.registers.set_hl(address);
    cpu.registers.set_carry(true);
    cpu.bus.write(address, value);

    cpu.step();

    assert_eq!(cpu.registers.get_a(), a.wrapping_sub(value).wrapping_sub(1));
}

#[test]
fn sbc_immediate() {
    let mut rng = rand::rng();

    let a = rng.random::<u8>();
    let value = rng.random::<u8>();

    let mut cpu = create_cpu(0xDE, Some(value), None);

    cpu.registers.set_a(a);
    cpu.registers.set_carry(false);

    cpu.step();

    assert_eq!(cpu.registers.get_a(), a.wrapping_sub(value));
}

#[test]
fn sbc_immediate_with_borrow() {
    let mut rng = rand::rng();

    let a = rng.random::<u8>();
    let value = rng.random::<u8>();

    let mut cpu = create_cpu(0xDE, Some(value), None);

    cpu.registers.set_a(a);
    cpu.registers.set_carry(true);

    cpu.step();

    assert_eq!(cpu.registers.get_a(), a.wrapping_sub(value).wrapping_sub(1));
}

#[test]
fn sbc_sets_zero_flag() {
    let mut cpu = create_cpu(0xDE, Some(1), None);

    cpu.registers.set_a(2);
    cpu.registers.set_carry(true);

    cpu.step();

    assert!(cpu.registers.get_zero());
}

#[test]
fn sbc_clears_zero_flag() {
    let mut rng = rand::rng();

    let value = rng.random::<u8>();
    let a = value.wrapping_add(1);

    let mut cpu = create_cpu(0xDE, Some(value), None);

    cpu.registers.set_a(a);
    cpu.registers.set_carry(false);

    cpu.step();

    assert!(!cpu.registers.get_zero());
}

#[test]
fn sbc_sets_subtract_flag() {
    let mut rng = rand::rng();

    let a = rng.random::<u8>();
    let value = rng.random::<u8>();

    let mut cpu = create_cpu(0xDE, Some(value), None);

    cpu.registers.set_a(a);
    cpu.registers.set_carry(rng.random());

    cpu.step();

    assert!(cpu.registers.get_subtract());
}

#[test]
fn sbc_sets_half_carry_with_borrow() {
    let mut cpu = create_cpu(0xDE, Some(1), None);

    cpu.registers.set_a(1);
    cpu.registers.set_carry(true);

    cpu.step();

    assert!(cpu.registers.get_half_carry());
}

#[test]
fn sbc_sets_carry_with_borrow() {
    let mut cpu = create_cpu(0xDE, Some(0), None);

    cpu.registers.set_a(0);
    cpu.registers.set_carry(true);

    cpu.step();

    assert!(cpu.registers.get_carry());
}

#[test]
fn sbc_register_advances_pc_by_one() {
    let mut rng = rand::rng();

    let registers = [
        (0x98, Register8::B),
        (0x99, Register8::C),
        (0x9A, Register8::D),
        (0x9B, Register8::E),
        (0x9C, Register8::H),
        (0x9D, Register8::L),
        (0x9F, Register8::A),
    ];

    let index = rng.random_range(0..registers.len());
    let (opcode, register) = registers[index];

    let mut cpu = create_cpu(opcode, None, None);

    cpu.registers.set_a(rng.random());
    cpu.set_register8(&register, rng.random());
    cpu.registers.set_carry(rng.random());

    let initial_pc = cpu.registers.get_pc();

    cpu.step();

    assert_eq!(cpu.registers.get_pc(), initial_pc + 1);
}

#[test]
fn sbc_from_hl_advances_pc_by_one() {
    let mut rng = rand::rng();
    let address = get_rand_wram_address();

    let mut cpu = create_cpu(0x9E, None, None);

    cpu.registers.set_a(rng.random());
    cpu.registers.set_hl(address);
    cpu.registers.set_carry(true);

    cpu.bus.write(address, 1);

    let initial_pc = cpu.registers.get_pc();

    cpu.step();

    assert_eq!(cpu.registers.get_pc(), initial_pc + 1);
}

#[test]
fn sbc_immediate_advances_pc_by_two() {
    let mut rng = rand::rng();

    let mut cpu = create_cpu(0xDE, Some(rng.random()), None);

    cpu.registers.set_a(rng.random());
    cpu.registers.set_carry(true);

    let initial_pc = cpu.registers.get_pc();

    cpu.step();

    assert_eq!(cpu.registers.get_pc(), initial_pc + 2);
}
