use super::*;

#[test]
fn inc_increments_all_registers() {
    let mut rng = rand::rng();

    let registers = [
        (0x04, Register8::B),
        (0x0C, Register8::C),
        (0x14, Register8::D),
        (0x1C, Register8::E),
        (0x24, Register8::H),
        (0x2C, Register8::L),
        (0x3C, Register8::A),
    ];

    for (opcode, register) in registers {
        let value: u8 = rng.random_range(0..=0xFE);

        let mut cpu = create_cpu(opcode, None, None);

        cpu.set_register8(&register, value);

        cpu.step();

        assert_eq!(cpu.get_register8(&register), value + 1);
    }
}

#[test]
fn inc_sets_zero_flag_when_result_is_zero() {
    let mut cpu = create_cpu(0x04, None, None);

    cpu.registers.set_b(0xFF);

    cpu.step();

    assert_eq!(cpu.registers.get_b(), 0x00);
    assert!(cpu.registers.get_zero());
}

#[test]
fn inc_clears_zero_flag_when_result_is_not_zero() {
    let mut rng = rand::rng();
    let value: u8 = rng.random_range(0..=0xFE);

    let mut cpu = create_cpu(0x04, None, None);

    cpu.registers.set_b(value);
    cpu.registers.set_zero(true);

    cpu.step();

    assert!(!cpu.registers.get_zero());
}

#[test]
fn inc_clears_subtract_flag() {
    let mut cpu = create_cpu(0x04, None, None);

    cpu.registers.set_b(0x05);
    cpu.registers.set_subtract(true);

    cpu.step();

    assert!(!cpu.registers.get_subtract());
}

#[test]
fn inc_sets_half_carry_when_lower_nibble_overflows() {
    let mut cpu = create_cpu(0x04, None, None);

    cpu.registers.set_b(0x0F);

    cpu.step();

    assert_eq!(cpu.registers.get_b(), 0x10);
    assert!(cpu.registers.get_half_carry());
}

#[test]
fn inc_clears_half_carry_when_lower_nibble_does_not_overflow() {
    let mut rng = rand::rng();
    let value: u8 = rng.random_range(0..=0x0E);

    let mut cpu = create_cpu(0x04, None, None);

    cpu.registers.set_b(value);
    cpu.registers.set_half_carry(true);

    cpu.step();

    assert!(!cpu.registers.get_half_carry());
}

#[test]
fn inc_preserves_carry_flag() {
    let mut rng = rand::rng();
    let value: u8 = rng.random();

    let mut cpu = create_cpu(0x04, None, None);

    cpu.registers.set_b(value);
    cpu.registers.set_carry(true);

    cpu.step();

    assert!(cpu.registers.get_carry());
}

#[test]
fn increment_16bit_increments_register() {
    let opcodes = [0x03, 0x13, 0x23, 0x33];

    for opcode in opcodes {
        let mut rng = rand::rng();
        let value: u16 = rng.random();

        let mut cpu = create_cpu(opcode, None, None);

        match opcode {
            0x03 => cpu.registers.set_bc(value),
            0x13 => cpu.registers.set_de(value),
            0x23 => cpu.registers.set_hl(value),
            0x33 => cpu.registers.set_sp(value),
            _ => unreachable!(),
        }

        cpu.step();

        let result = match opcode {
            0x03 => cpu.registers.get_bc(),
            0x13 => cpu.registers.get_de(),
            0x23 => cpu.registers.get_hl(),
            0x33 => cpu.registers.get_sp(),
            _ => unreachable!(),
        };

        assert_eq!(result, value.wrapping_add(1));
    }
}

#[test]
fn increment_16bit_wraps_around() {
    let opcodes = [0x03, 0x13, 0x23, 0x33];

    for opcode in opcodes {
        let mut cpu = create_cpu(opcode, None, None);

        match opcode {
            0x03 => cpu.registers.set_bc(u16::MAX),
            0x13 => cpu.registers.set_de(u16::MAX),
            0x23 => cpu.registers.set_hl(u16::MAX),
            0x33 => cpu.registers.set_sp(u16::MAX),
            _ => unreachable!(),
        }

        cpu.step();

        let result = match opcode {
            0x03 => cpu.registers.get_bc(),
            0x13 => cpu.registers.get_de(),
            0x23 => cpu.registers.get_hl(),
            0x33 => cpu.registers.get_sp(),
            _ => unreachable!(),
        };

        assert_eq!(result, 0);
    }
}

#[test]
fn increment_16bit_advances_pc_by_one() {
    let opcodes = [0x03, 0x13, 0x23, 0x33];

    for opcode in opcodes {
        let mut rng = rand::rng();
        let value: u16 = rng.random();

        let mut cpu = create_cpu(opcode, None, None);

        match opcode {
            0x03 => cpu.registers.set_bc(value),
            0x13 => cpu.registers.set_de(value),
            0x23 => cpu.registers.set_hl(value),
            0x33 => cpu.registers.set_sp(value),
            _ => unreachable!(),
        }

        cpu.step();

        assert_eq!(cpu.registers.get_pc(), GAME_ENTRY_POINT + 1);
    }
}

#[test]
fn increment_16bit_preserves_flags() {
    let opcodes = [0x03, 0x13, 0x23, 0x33];

    for opcode in opcodes {
        let mut rng = rand::rng();
        let value: u16 = rng.random();

        let mut cpu = create_cpu(opcode, None, None);

        cpu.registers.set_zero(true);
        cpu.registers.set_subtract(true);
        cpu.registers.set_half_carry(true);
        cpu.registers.set_carry(true);

        match opcode {
            0x03 => cpu.registers.set_bc(value),
            0x13 => cpu.registers.set_de(value),
            0x23 => cpu.registers.set_hl(value),
            0x33 => cpu.registers.set_sp(value),
            _ => unreachable!(),
        }

        cpu.step();

        assert!(cpu.registers.get_zero());
        assert!(cpu.registers.get_subtract());
        assert!(cpu.registers.get_half_carry());
        assert!(cpu.registers.get_carry());
    }
}
