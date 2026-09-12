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

#[test]
fn add_register8() {
    let mut rng = rand::rng();

    let opcodes = [
        0x80, // ADD A, B
        0x81, // ADD A, C
        0x82, // ADD A, D
        0x83, // ADD A, E
        0x84, // ADD A, H
        0x85, // ADD A, L
        0x87, // ADD A, A
    ];

    for opcode in opcodes {
        let mut cpu = create_cpu(opcode, None, None);

        let a = rng.random();
        let value = rng.random();

        cpu.registers.set_a(a);

        match opcode {
            0x80 => cpu.registers.set_b(value),
            0x81 => cpu.registers.set_c(value),
            0x82 => cpu.registers.set_d(value),
            0x83 => cpu.registers.set_e(value),
            0x84 => cpu.registers.set_h(value),
            0x85 => cpu.registers.set_l(value),
            0x87 => {}
            _ => unreachable!(),
        }

        let expected = if opcode == 0x87 {
            a.wrapping_add(a)
        } else {
            a.wrapping_add(value)
        };

        cpu.step();

        assert_eq!(cpu.registers.get_a(), expected);
    }
}

#[test]
fn add_register8_sets_zero_flag() {
    let mut cpu = create_cpu(0x80, None, None);

    cpu.registers.set_a(0);
    cpu.registers.set_b(0);

    cpu.step();

    assert!(cpu.registers.get_zero());
}

#[test]
fn add_register8_clears_subtract_flag() {
    let mut cpu = create_cpu(0x80, None, None);

    cpu.registers.set_subtract(true);

    cpu.step();

    assert!(!cpu.registers.get_subtract());
}

#[test]
fn add_register8_sets_half_carry() {
    let mut cpu = create_cpu(0x80, None, None);

    cpu.registers.set_a(0x0F);
    cpu.registers.set_b(1);

    cpu.step();

    assert!(cpu.registers.get_half_carry());
}

#[test]
fn add_register8_sets_carry() {
    let mut cpu = create_cpu(0x80, None, None);

    cpu.registers.set_a(u8::MAX);
    cpu.registers.set_b(1);

    cpu.step();

    assert!(cpu.registers.get_carry());
}

#[test]
fn add_register8_advances_pc_by_one() {
    let mut cpu = create_cpu(0x80, None, None);

    cpu.step();

    assert_eq!(cpu.registers.get_pc(), GAME_ENTRY_POINT + 1);
}

#[test]
fn add_from_hl() {
    let mut rng = rand::rng();

    let mut cpu = create_cpu(0x86, None, None);

    let address = get_rand_wram_address();
    let a = rng.random();
    let value = rng.random();

    // HL -> Address
    cpu.registers.set_hl(address);

    // A -> Value
    cpu.registers.set_a(a);

    // Memory[Address] -> Value
    cpu.bus.write(address, value);

    // ADD A, (HL)
    cpu.step();

    // A -> A + Value
    assert_eq!(cpu.registers.get_a(), a.wrapping_add(value));
}

#[test]
fn add_from_hl_sets_zero_flag() {
    let mut cpu = create_cpu(0x86, None, None);

    let address = get_rand_wram_address();

    cpu.registers.set_hl(address);
    cpu.registers.set_a(0);
    cpu.bus.write(address, 0);

    cpu.step();

    assert!(cpu.registers.get_zero());
}

#[test]
fn add_from_hl_clears_subtract_flag() {
    let mut cpu = create_cpu(0x86, None, None);

    let address = get_rand_wram_address();

    cpu.registers.set_hl(address);
    cpu.registers.set_subtract(true);

    cpu.step();

    assert!(!cpu.registers.get_subtract());
}

#[test]
fn add_from_hl_sets_half_carry() {
    let mut cpu = create_cpu(0x86, None, None);

    let address = get_rand_wram_address();

    cpu.registers.set_hl(address);
    cpu.registers.set_a(0x0F);
    cpu.bus.write(address, 1);

    cpu.step();

    assert!(cpu.registers.get_half_carry());
}

#[test]
fn add_from_hl_sets_carry() {
    let mut cpu = create_cpu(0x86, None, None);

    let address = get_rand_wram_address();

    cpu.registers.set_hl(address);
    cpu.registers.set_a(u8::MAX);
    cpu.bus.write(address, 1);

    cpu.step();

    assert!(cpu.registers.get_carry());
}

#[test]
fn add_from_hl_advances_pc_by_one() {
    let mut cpu = create_cpu(0x86, None, None);

    cpu.step();

    assert_eq!(cpu.registers.get_pc(), GAME_ENTRY_POINT + 1);
}

#[test]
fn add_immediate() {
    let mut rng = rand::rng();

    let value = rng.random();
    let mut cpu = create_cpu(0xC6, Some(value), None);

    let a = rng.random();

    // A -> Value
    cpu.registers.set_a(a);

    // ADD A, n
    cpu.step();

    // A -> A + Value
    assert_eq!(cpu.registers.get_a(), a.wrapping_add(value));
}

#[test]
fn add_immediate_sets_zero_flag() {
    let mut cpu = create_cpu(0xC6, Some(0), None);

    cpu.registers.set_a(0);

    cpu.step();

    assert!(cpu.registers.get_zero());
}

#[test]
fn add_immediate_clears_subtract_flag() {
    let mut cpu = create_cpu(0xC6, Some(1), None);

    cpu.registers.set_subtract(true);

    cpu.step();

    assert!(!cpu.registers.get_subtract());
}

#[test]
fn add_immediate_sets_half_carry() {
    let mut cpu = create_cpu(0xC6, Some(1), None);

    cpu.registers.set_a(0x0F);

    cpu.step();

    assert!(cpu.registers.get_half_carry());
}

#[test]
fn add_immediate_sets_carry() {
    let mut cpu = create_cpu(0xC6, Some(1), None);

    cpu.registers.set_a(u8::MAX);

    cpu.step();

    assert!(cpu.registers.get_carry());
}

#[test]
fn add_immediate_advances_pc_by_two() {
    let mut cpu = create_cpu(0xC6, Some(1), None);

    cpu.step();

    assert_eq!(cpu.registers.get_pc(), GAME_ENTRY_POINT + 2);
}
