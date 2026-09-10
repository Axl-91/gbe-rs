use super::*;

#[test]
fn load8_from_bc_loads_value_into_a() {
    let mut rng = rand::rng();

    let value: u8 = rng.random();
    let address = 0xC000;

    let mut cpu = create_cpu(0x0A, None, None);

    // BC -> 0xC000
    cpu.registers.set_bc(address);

    // (0xC000) -> Value
    cpu.bus.write(address, value);

    // LD A, (BC)
    cpu.step();

    // A == Value
    assert_eq!(cpu.registers.get_a(), value);
}

#[test]
fn load8_from_bc_advances_pc_by_one() {
    let mut cpu = create_cpu(0x0A, None, None);

    cpu.registers.set_bc(0xC000);

    cpu.step();

    assert_eq!(cpu.registers.get_pc(), GAME_ENTRY_POINT + 1);
}

#[test]
fn load8_from_bc_preserves_flags() {
    let mut cpu = create_cpu(0x0A, None, None);

    cpu.registers.set_bc(0xC000);

    cpu.registers.set_zero(true);
    cpu.registers.set_subtract(true);
    cpu.registers.set_half_carry(true);
    cpu.registers.set_carry(true);

    cpu.step();

    assert!(cpu.registers.get_zero());
    assert!(cpu.registers.get_subtract());
    assert!(cpu.registers.get_half_carry());
    assert!(cpu.registers.get_carry());
}

#[test]
fn load8_from_de_loads_value_into_a() {
    let mut rng = rand::rng();

    let value: u8 = rng.random();
    let address = 0xC000;

    let mut cpu = create_cpu(0x1A, None, None);

    // DE -> 0xC000
    cpu.registers.set_de(address);

    // (0xC000) -> Value
    cpu.bus.write(address, value);

    // LD A, (DE)
    cpu.step();

    // A == Value
    assert_eq!(cpu.registers.get_a(), value);
}

#[test]
fn load8_from_de_advances_pc_by_one() {
    let mut cpu = create_cpu(0x1A, None, None);

    cpu.registers.set_de(0xC000);

    cpu.step();

    assert_eq!(cpu.registers.get_pc(), GAME_ENTRY_POINT + 1);
}

#[test]
fn load8_from_de_preserves_flags() {
    let mut cpu = create_cpu(0x1A, None, None);

    cpu.registers.set_de(0xC000);

    cpu.registers.set_zero(true);
    cpu.registers.set_subtract(true);
    cpu.registers.set_half_carry(true);
    cpu.registers.set_carry(true);

    cpu.step();

    assert!(cpu.registers.get_zero());
    assert!(cpu.registers.get_subtract());
    assert!(cpu.registers.get_half_carry());
    assert!(cpu.registers.get_carry());
}

#[test]
fn load8_to_bc_writes_a_to_memory() {
    let mut rng = rand::rng();

    let value: u8 = rng.random();
    let address = 0xC000;

    let mut cpu = create_cpu(0x02, None, None);

    // A -> Value
    cpu.registers.set_a(value);

    // BC -> 0xC000
    cpu.registers.set_bc(address);

    // LD (BC), A
    cpu.step();

    // (0xC000) == Value
    assert_eq!(cpu.bus.read(address), value);
}

#[test]
fn load8_to_bc_advances_pc_by_one() {
    let mut cpu = create_cpu(0x02, None, None);

    cpu.registers.set_bc(0xC000);

    cpu.step();

    assert_eq!(cpu.registers.get_pc(), GAME_ENTRY_POINT + 1);
}

#[test]
fn load8_to_bc_preserves_flags() {
    let mut cpu = create_cpu(0x02, None, None);

    cpu.registers.set_bc(0xC000);

    cpu.registers.set_zero(true);
    cpu.registers.set_subtract(true);
    cpu.registers.set_half_carry(true);
    cpu.registers.set_carry(true);

    cpu.step();

    assert!(cpu.registers.get_zero());
    assert!(cpu.registers.get_subtract());
    assert!(cpu.registers.get_half_carry());
    assert!(cpu.registers.get_carry());
}

#[test]
fn load8_to_de_writes_a_to_memory() {
    let mut rng = rand::rng();

    let value: u8 = rng.random();
    let address = 0xC000;

    let mut cpu = create_cpu(0x12, None, None);

    // A -> Value
    cpu.registers.set_a(value);

    // DE -> 0xC000
    cpu.registers.set_de(address);

    // LD (DE), A
    cpu.step();

    // (0xC000) == Value
    assert_eq!(cpu.bus.read(address), value);
}

#[test]
fn load8_to_de_advances_pc_by_one() {
    let mut cpu = create_cpu(0x12, None, None);

    cpu.registers.set_de(0xC000);

    cpu.step();

    assert_eq!(cpu.registers.get_pc(), GAME_ENTRY_POINT + 1);
}

#[test]
fn load8_to_de_preserves_flags() {
    let mut cpu = create_cpu(0x12, None, None);

    cpu.registers.set_de(0xC000);

    cpu.registers.set_zero(true);
    cpu.registers.set_subtract(true);
    cpu.registers.set_half_carry(true);
    cpu.registers.set_carry(true);

    cpu.step();

    assert!(cpu.registers.get_zero());
    assert!(cpu.registers.get_subtract());
    assert!(cpu.registers.get_half_carry());
    assert!(cpu.registers.get_carry());
}

#[test]
fn load8_from_address_loads_value_into_a() {
    let mut rng = rand::rng();

    let value: u8 = rng.random();
    let address = 0xC000;
    let lower_bits_address = (address & 0x00FF) as u8;
    let higher_bits_address = (address >> 8) as u8;

    let mut cpu = create_cpu(0xFA, Some(lower_bits_address), Some(higher_bits_address));

    // (0xC000) -> Value
    cpu.bus.write(address, value);

    // LD A, (a16)
    cpu.step();

    // A == Value
    assert_eq!(cpu.registers.get_a(), value);
}

#[test]
fn load8_from_address_advances_pc_by_three() {
    let address = 0xC000;
    let lower_bits_address = (address & 0x00FF) as u8;
    let higher_bits_address = (address >> 8) as u8;

    let mut cpu = create_cpu(0xFA, Some(lower_bits_address), Some(higher_bits_address));

    cpu.bus.write(GAME_ENTRY_POINT + 2, (address >> 8) as u8);

    cpu.step();

    assert_eq!(cpu.registers.get_pc(), GAME_ENTRY_POINT + 3);
}

#[test]
fn load8_from_address_preserves_flags() {
    let address = 0xC000;
    let lower_bits_address = (address & 0x00FF) as u8;
    let higher_bits_address = (address >> 8) as u8;

    let mut cpu = create_cpu(0xFA, Some(lower_bits_address), Some(higher_bits_address));

    cpu.bus.write(GAME_ENTRY_POINT + 2, (address >> 8) as u8);

    cpu.registers.set_zero(true);
    cpu.registers.set_subtract(true);
    cpu.registers.set_half_carry(true);
    cpu.registers.set_carry(true);

    cpu.step();

    assert!(cpu.registers.get_zero());
    assert!(cpu.registers.get_subtract());
    assert!(cpu.registers.get_half_carry());
    assert!(cpu.registers.get_carry());
}

#[test]
fn load8_to_address_writes_a_to_memory() {
    let mut rng = rand::rng();

    let value: u8 = rng.random();
    let address = 0xC000;
    let lower_bits_address = (address & 0x00FF) as u8;
    let higher_bits_address = (address >> 8) as u8;

    let mut cpu = create_cpu(0xEA, Some(lower_bits_address), Some(higher_bits_address));

    // A -> Value
    cpu.registers.set_a(value);

    // Low byte of address -> 0x0101
    cpu.bus
        .write(GAME_ENTRY_POINT + 1, (address & 0x00FF) as u8);

    // High byte of address -> 0x0102
    cpu.bus.write(GAME_ENTRY_POINT + 2, (address >> 8) as u8);

    // LD (a16), A
    cpu.step();

    // (0xC000) == Value
    assert_eq!(cpu.bus.read(address), value);
}

#[test]
fn load8_to_address_advances_pc_by_three() {
    let address = 0xC000;
    let lower_bits_address = (address & 0x00FF) as u8;
    let higher_bits_address = (address >> 8) as u8;

    let mut cpu = create_cpu(0xEA, Some(lower_bits_address), Some(higher_bits_address));

    cpu.bus.write(GAME_ENTRY_POINT + 2, (address >> 8) as u8);

    cpu.step();

    assert_eq!(cpu.registers.get_pc(), GAME_ENTRY_POINT + 3);
}

#[test]
fn load8_to_address_preserves_flags() {
    let address = 0xC000;
    let lower_bits_address = (address & 0x00FF) as u8;
    let higher_bits_address = (address >> 8) as u8;

    let mut cpu = create_cpu(0xEA, Some(lower_bits_address), Some(higher_bits_address));

    cpu.bus.write(GAME_ENTRY_POINT + 2, (address >> 8) as u8);

    cpu.registers.set_zero(true);
    cpu.registers.set_subtract(true);
    cpu.registers.set_half_carry(true);
    cpu.registers.set_carry(true);

    cpu.step();

    assert!(cpu.registers.get_zero());
    assert!(cpu.registers.get_subtract());
    assert!(cpu.registers.get_half_carry());
    assert!(cpu.registers.get_carry());
}
