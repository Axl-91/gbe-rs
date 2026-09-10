use super::*;

#[test]
fn load16_to_address_writes_sp_to_memory() {
    let mut rng = rand::rng();

    let address = get_rand_wram_address();
    let value: u16 = rng.random();

    let low_address = address as u8;
    let high_address = (address >> 8) as u8;

    let mut cpu = create_cpu(0x08, Some(low_address), Some(high_address));

    // SP -> Value
    cpu.registers.set_sp(value);

    // LD (a16), SP
    cpu.step();

    // Low byte of Value -> Memory[address]
    assert_eq!(cpu.bus.read(address), value as u8);

    // High byte of Value -> Memory[address + 1]
    assert_eq!(cpu.bus.read(address + 1), (value >> 8) as u8);
}

#[test]
fn load16_to_address_advances_pc_by_three() {
    let mut rng = rand::rng();

    let address = get_rand_wram_address();
    let low_address = address as u8;
    let high_address = (address >> 8) as u8;

    let mut cpu = create_cpu(0x08, Some(low_address), Some(high_address));

    cpu.registers.set_sp(rng.random());

    // LD (a16), SP
    cpu.step();

    assert_eq!(cpu.registers.get_pc(), GAME_ENTRY_POINT + 3);
}

#[test]
fn load16_to_address_preserves_flags() {
    let mut rng = rand::rng();

    let address = get_rand_wram_address();
    let low_address = address as u8;
    let high_address = (address >> 8) as u8;

    let mut cpu = create_cpu(0x08, Some(low_address), Some(high_address));

    cpu.registers.set_sp(rng.random());

    cpu.registers.set_zero(true);
    cpu.registers.set_subtract(true);
    cpu.registers.set_half_carry(true);
    cpu.registers.set_carry(true);

    // LD (a16), SP
    cpu.step();

    assert!(cpu.registers.get_zero());
    assert!(cpu.registers.get_subtract());
    assert!(cpu.registers.get_half_carry());
    assert!(cpu.registers.get_carry());
}

#[test]
fn load_sp_from_hl_loads_hl_into_sp() {
    let mut rng = rand::rng();

    let value: u16 = rng.random();

    let mut cpu = create_cpu(0xF9, None, None);

    // HL -> Value
    cpu.registers.set_hl(value);

    // LD SP, HL
    cpu.step();

    // SP -> Value
    assert_eq!(cpu.registers.get_sp(), value);
}

#[test]
fn load_sp_from_hl_preserves_pc() {
    let mut rng = rand::rng();

    let value: u16 = rng.random();

    let mut cpu = create_cpu(0xF9, None, None);

    cpu.registers.set_hl(value);

    // LD SP, HL
    cpu.step();

    assert_eq!(cpu.registers.get_pc(), GAME_ENTRY_POINT + 1);
}

#[test]
fn load_sp_from_hl_preserves_flags() {
    let mut rng = rand::rng();

    let value: u16 = rng.random();

    let mut cpu = create_cpu(0xF9, None, None);

    cpu.registers.set_hl(value);

    cpu.registers.set_zero(true);
    cpu.registers.set_subtract(true);
    cpu.registers.set_half_carry(true);
    cpu.registers.set_carry(true);

    // LD SP, HL
    cpu.step();

    assert!(cpu.registers.get_zero());
    assert!(cpu.registers.get_subtract());
    assert!(cpu.registers.get_half_carry());
    assert!(cpu.registers.get_carry());
}
