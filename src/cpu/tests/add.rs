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
