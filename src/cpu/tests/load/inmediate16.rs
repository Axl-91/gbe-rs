use super::*;

#[test]
fn load16_immediate_loads_value_into_bc() {
    let mut cpu = create_cpu(0x01, Some(0x34), Some(0x12));

    cpu.step();

    assert_eq!(cpu.registers.get_bc(), 0x1234);
}

#[test]
fn load16_immediate_loads_value_into_de() {
    let mut cpu = create_cpu(0x11, Some(0x34), Some(0x12));

    cpu.step();

    assert_eq!(cpu.registers.get_de(), 0x1234);
}

#[test]
fn load16_immediate_loads_value_into_hl() {
    let mut cpu = create_cpu(0x21, Some(0x34), Some(0x12));

    cpu.step();

    assert_eq!(cpu.registers.get_hl(), 0x1234);
}

#[test]
fn load16_immediate_loads_value_into_sp() {
    let mut cpu = create_cpu(0x31, Some(0x34), Some(0x12));

    cpu.step();

    assert_eq!(cpu.registers.get_sp(), 0x1234);
}

#[test]
fn load16_immediate_advances_pc_by_three() {
    let opcodes = [0x01, 0x11, 0x21, 0x31];

    for opcode in opcodes {
        let mut cpu = create_cpu(opcode, Some(0x34), Some(0x12));

        cpu.step();

        assert_eq!(cpu.registers.get_pc(), GAME_ENTRY_POINT + 3);
    }
}

#[test]
fn load16_immediate_preserves_flags() {
    let opcodes = [0x01, 0x11, 0x21, 0x31];

    for opcode in opcodes {
        let mut cpu = create_cpu(opcode, Some(0x34), Some(0x12));

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
}

#[test]
fn load_hl_from_sp_plus_immediate_loads_positive_offset() {
    let mut rng = rand::rng();

    let sp = rng.random_range(WRAM_START..WRAM_END);
    let offset = rng.random_range(1..=i8::MAX);

    let mut cpu = create_cpu(0xF8, Some(offset as u8), None);

    // SP -> Value
    cpu.registers.set_sp(sp);

    // LD HL, SP + e8
    cpu.step();

    // SP + offset -> HL
    let expected = (sp as i16 + offset as i16) as u16;

    assert_eq!(cpu.registers.get_hl(), expected);
}

#[test]
fn load_hl_from_sp_plus_immediate_loads_negative_offset() {
    let mut rng = rand::rng();

    let sp = rng.random_range(WRAM_START..WRAM_END);
    let offset = rng.random_range(i8::MIN..=-1);

    let mut cpu = create_cpu(0xF8, Some(offset as u8), None);

    // SP -> Value
    cpu.registers.set_sp(sp);

    // LD HL, SP + e8
    cpu.step();

    // SP + offset -> HL
    let expected = (sp as i16 + offset as i16) as u16;

    assert_eq!(cpu.registers.get_hl(), expected);
}

#[test]
fn load_hl_from_sp_plus_immediate_advances_pc_by_two() {
    let mut rng = rand::rng();

    let offset: i8 = rng.random();

    let mut cpu = create_cpu(0xF8, Some(offset as u8), None);

    cpu.registers.set_sp(rng.random());

    // LD HL, SP + e8
    cpu.step();

    assert_eq!(cpu.registers.get_pc(), GAME_ENTRY_POINT + 2);
}

#[test]
fn load_hl_from_sp_plus_immediate_sets_half_carry() {
    let mut rng = rand::rng();

    let sp = rng.random_range(WRAM_START..WRAM_END) | 0x000F;
    let offset = 1i8;

    let mut cpu = create_cpu(0xF8, Some(offset as u8), None);

    cpu.registers.set_sp(sp);

    // LD HL, SP + e8
    cpu.step();

    assert!(cpu.registers.get_half_carry());
}

#[test]
fn load_hl_from_sp_plus_immediate_sets_carry() {
    let mut rng = rand::rng();

    let sp = rng.random_range(WRAM_START..WRAM_END) | 0x00FF;
    let offset = 1i8;

    let mut cpu = create_cpu(0xF8, Some(offset as u8), None);

    cpu.registers.set_sp(sp);

    // LD HL, SP + e8
    cpu.step();

    assert!(cpu.registers.get_carry());
}

#[test]
fn load_hl_from_sp_plus_immediate_sets_correspondant_flags() {
    let mut rng = rand::rng();

    let sp: u16 = rng.random();
    let offset: i8 = rng.random();
    let offset_u16 = offset as u8 as u16;

    let mut cpu = create_cpu(0xF8, Some(offset as u8), None);

    cpu.registers.set_sp(sp);

    // Set flags beforehand to verify that Z and N are cleared.
    cpu.registers.set_zero(true);
    cpu.registers.set_subtract(true);

    // LD HL, SP + e8
    cpu.step();

    assert!(!cpu.registers.get_zero());
    assert!(!cpu.registers.get_subtract());

    let expected_half_carry = (sp & 0x000F) + (offset_u16 & 0x000F) > 0x000F;

    let expected_carry = (sp & 0x00FF) + offset_u16 > 0x00FF;

    assert_eq!(cpu.registers.get_half_carry(), expected_half_carry);
    assert_eq!(cpu.registers.get_carry(), expected_carry);
}
