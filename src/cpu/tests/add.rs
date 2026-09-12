use super::*;

#[test]
fn add_hl_register16() {
    let mut rng = rand::rng();

    let opcodes = [
        0x09, // ADD HL, BC
        0x19, // ADD HL, DE
        0x29, // ADD HL, HL
        0x39, // ADD HL, SP
    ];

    for opcode in opcodes {
        let mut cpu = create_cpu(opcode, None, None);

        let hl = rng.random();
        let value = rng.random();

        cpu.registers.set_hl(hl);

        match opcode {
            0x09 => cpu.registers.set_bc(value),
            0x19 => cpu.registers.set_de(value),
            0x29 => {}
            0x39 => cpu.registers.set_sp(value),
            _ => unreachable!(),
        }

        let expected = if opcode == 0x29 {
            hl.wrapping_add(hl)
        } else {
            hl.wrapping_add(value)
        };

        cpu.step();

        assert_eq!(cpu.registers.get_hl(), expected);
    }
}

#[test]
fn add_hl_register16_wraps_around() {
    let mut cpu = create_cpu(0x09, None, None);

    cpu.registers.set_hl(u16::MAX);
    cpu.registers.set_bc(1);

    cpu.step();

    assert_eq!(cpu.registers.get_hl(), 0);
}

#[test]
fn add_hl_register16_advances_pc_by_one() {
    let mut rng = rand::rng();
    let mut cpu = create_cpu(0x09, None, None);

    cpu.registers.set_hl(rng.random());
    cpu.registers.set_bc(rng.random());

    cpu.step();

    assert_eq!(cpu.registers.get_pc(), GAME_ENTRY_POINT + 1);
}

#[test]
fn add_hl_register16_preserves_zero_flag() {
    let mut cpu = create_cpu(0x09, None, None);

    cpu.registers.set_hl(1);
    cpu.registers.set_bc(1);
    cpu.registers.set_zero(true);

    cpu.step();

    assert!(cpu.registers.get_zero());
}

#[test]
fn add_hl_register16_clears_subtract_flag() {
    let mut cpu = create_cpu(0x09, None, None);

    cpu.registers.set_hl(1);
    cpu.registers.set_bc(1);
    cpu.registers.set_subtract(true);

    cpu.step();

    assert!(!cpu.registers.get_subtract());
}

#[test]
fn add_hl_register16_sets_half_carry() {
    let mut cpu = create_cpu(0x09, None, None);

    cpu.registers.set_hl(0x0FFF);
    cpu.registers.set_bc(1);

    cpu.step();

    assert!(cpu.registers.get_half_carry());
}

#[test]
fn add_hl_register16_sets_carry() {
    let mut cpu = create_cpu(0x09, None, None);

    cpu.registers.set_hl(u16::MAX);
    cpu.registers.set_bc(1);

    cpu.step();

    assert!(cpu.registers.get_carry());
}

#[test]
fn add_sp_immediate() {
    let mut rng = rand::rng();
    let mut cpu = create_cpu(0xE8, Some(rng.random()), None);

    let sp = rng.random_range(0x0100..=0xFEFF);
    let offset = cpu.bus.read(GAME_ENTRY_POINT + 1) as i8;

    cpu.registers.set_sp(sp);

    let expected = (sp as i16 + offset as i16) as u16;

    cpu.step();

    assert_eq!(cpu.registers.get_sp(), expected);
}

#[test]
fn add_sp_immediate_with_negative_offset() {
    let mut rng = rand::rng();
    let offset = rng.random_range(i8::MIN..=-1);

    let mut cpu = create_cpu(0xE8, Some(offset as u8), None);

    let sp = rng.random_range(0x0100..=u16::MAX);

    cpu.registers.set_sp(sp);

    let expected = (sp as i16 + offset as i16) as u16;

    cpu.step();

    assert_eq!(cpu.registers.get_sp(), expected);
}

#[test]
fn add_sp_immediate_advances_pc_by_two() {
    let mut rng = rand::rng();
    let mut cpu = create_cpu(0xE8, Some(rng.random()), None);

    cpu.registers.set_sp(rng.random());

    cpu.step();

    assert_eq!(cpu.registers.get_pc(), GAME_ENTRY_POINT + 2);
}

#[test]
fn add_sp_immediate_clears_zero_flag() {
    let mut rng = rand::rng();
    let mut cpu = create_cpu(0xE8, Some(rng.random()), None);

    cpu.registers.set_sp(rng.random());
    cpu.registers.set_zero(true);

    cpu.step();

    assert!(!cpu.registers.get_zero());
}

#[test]
fn add_sp_immediate_clears_subtract_flag() {
    let mut rng = rand::rng();
    let mut cpu = create_cpu(0xE8, Some(rng.random()), None);

    cpu.registers.set_sp(rng.random());
    cpu.registers.set_subtract(true);

    cpu.step();

    assert!(!cpu.registers.get_subtract());
}

#[test]
fn add_sp_immediate_sets_half_carry() {
    let mut cpu = create_cpu(0xE8, Some(1), None);

    cpu.registers.set_sp(0x000F);

    cpu.step();

    assert!(cpu.registers.get_half_carry());
}

#[test]
fn add_sp_immediate_sets_carry() {
    let mut cpu = create_cpu(0xE8, Some(1), None);

    cpu.registers.set_sp(0x00FF);

    cpu.step();

    assert!(cpu.registers.get_carry());
}

#[test]
fn add_sp_immediate_wraps_around() {
    let mut cpu = create_cpu(0xE8, Some(1), None);

    cpu.registers.set_sp(u16::MAX);

    cpu.step();

    assert_eq!(cpu.registers.get_sp(), 0);
}
