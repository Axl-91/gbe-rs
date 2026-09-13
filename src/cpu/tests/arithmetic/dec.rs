use super::*;

#[test]
fn dec_decrements_all_registers() {
    let mut rng = rand::rng();

    let registers = [
        (0x05, Register8::B),
        (0x0D, Register8::C),
        (0x15, Register8::D),
        (0x1D, Register8::E),
        (0x25, Register8::H),
        (0x2D, Register8::L),
        (0x3D, Register8::A),
    ];

    for (opcode, register) in registers {
        let value: u8 = rng.random_range(1..=u8::MAX);

        let mut cpu = create_cpu(opcode, None, None);

        cpu.set_register8(&register, value);

        cpu.step();

        assert_eq!(cpu.get_register8(&register), value - 1);
    }
}

#[test]
fn dec_sets_zero_flag_when_result_is_zero() {
    let mut cpu = create_cpu(0x05, None, None);

    cpu.registers.set_b(1);

    cpu.step();

    assert_eq!(cpu.registers.get_b(), 0);
    assert!(cpu.registers.get_zero());
}

#[test]
fn dec_clears_zero_flag_when_result_is_not_zero() {
    let mut rng = rand::rng();
    let value: u8 = rng.random_range(2..=u8::MAX);

    let mut cpu = create_cpu(0x05, None, None);

    cpu.registers.set_b(value);
    cpu.registers.set_zero(true);

    cpu.step();

    assert!(!cpu.registers.get_zero());
}

#[test]
fn dec_sets_subtract_flag() {
    let mut cpu = create_cpu(0x05, None, None);

    cpu.registers.set_b(5);
    cpu.registers.set_subtract(false);

    cpu.step();

    assert!(cpu.registers.get_subtract());
}

#[test]
fn dec_sets_half_carry_when_lower_nibble_borrows() {
    let mut cpu = create_cpu(0x05, None, None);

    cpu.registers.set_b(0x10);

    cpu.step();

    assert_eq!(cpu.registers.get_b(), 0x0F);
    assert!(cpu.registers.get_half_carry());
}

#[test]
fn dec_clears_half_carry_when_lower_nibble_does_not_borrow() {
    let mut rng = rand::rng();
    let value: u8 = rng.random_range(1..=0x0F);

    let mut cpu = create_cpu(0x05, None, None);

    cpu.registers.set_b(value);
    cpu.registers.set_half_carry(true);

    cpu.step();

    assert!(!cpu.registers.get_half_carry());
}

#[test]
fn dec_preserves_carry_flag() {
    let mut rng = rand::rng();
    let value: u8 = rng.random();

    let mut cpu = create_cpu(0x05, None, None);

    cpu.registers.set_b(value);
    cpu.registers.set_carry(true);

    cpu.step();

    assert!(cpu.registers.get_carry());
}

#[test]
fn decrement_16bit_decrements_register() {
    let opcodes = [0x0B, 0x1B, 0x2B, 0x3B];

    for opcode in opcodes {
        let mut rng = rand::rng();
        let value: u16 = rng.random();

        let mut cpu = create_cpu(opcode, None, None);

        match opcode {
            0x0B => cpu.registers.set_bc(value),
            0x1B => cpu.registers.set_de(value),
            0x2B => cpu.registers.set_hl(value),
            0x3B => cpu.registers.set_sp(value),
            _ => unreachable!(),
        }

        cpu.step();

        let result = match opcode {
            0x0B => cpu.registers.get_bc(),
            0x1B => cpu.registers.get_de(),
            0x2B => cpu.registers.get_hl(),
            0x3B => cpu.registers.get_sp(),
            _ => unreachable!(),
        };

        assert_eq!(result, value.wrapping_sub(1));
    }
}

#[test]
fn decrement_16bit_wraps_around() {
    let opcodes = [0x0B, 0x1B, 0x2B, 0x3B];

    for opcode in opcodes {
        let mut cpu = create_cpu(opcode, None, None);

        match opcode {
            0x0B => cpu.registers.set_bc(0),
            0x1B => cpu.registers.set_de(0),
            0x2B => cpu.registers.set_hl(0),
            0x3B => cpu.registers.set_sp(0),
            _ => unreachable!(),
        }

        cpu.step();

        let result = match opcode {
            0x0B => cpu.registers.get_bc(),
            0x1B => cpu.registers.get_de(),
            0x2B => cpu.registers.get_hl(),
            0x3B => cpu.registers.get_sp(),
            _ => unreachable!(),
        };

        assert_eq!(result, u16::MAX);
    }
}

#[test]
fn decrement_16bit_advances_pc_by_one() {
    let opcodes = [0x0B, 0x1B, 0x2B, 0x3B];

    for opcode in opcodes {
        let mut rng = rand::rng();
        let value: u16 = rng.random();

        let mut cpu = create_cpu(opcode, None, None);

        match opcode {
            0x0B => cpu.registers.set_bc(value),
            0x1B => cpu.registers.set_de(value),
            0x2B => cpu.registers.set_hl(value),
            0x3B => cpu.registers.set_sp(value),
            _ => unreachable!(),
        }

        cpu.step();

        assert_eq!(cpu.registers.get_pc(), GAME_ENTRY_POINT + 1);
    }
}

#[test]
fn decrement_16bit_preserves_flags() {
    let opcodes = [0x0B, 0x1B, 0x2B, 0x3B];

    for opcode in opcodes {
        let mut rng = rand::rng();
        let value: u16 = rng.random();

        let mut cpu = create_cpu(opcode, None, None);

        cpu.registers.set_zero(true);
        cpu.registers.set_subtract(true);
        cpu.registers.set_half_carry(true);
        cpu.registers.set_carry(true);

        match opcode {
            0x0B => cpu.registers.set_bc(value),
            0x1B => cpu.registers.set_de(value),
            0x2B => cpu.registers.set_hl(value),
            0x3B => cpu.registers.set_sp(value),
            _ => unreachable!(),
        }

        cpu.step();

        assert!(cpu.registers.get_zero());
        assert!(cpu.registers.get_subtract());
        assert!(cpu.registers.get_half_carry());
        assert!(cpu.registers.get_carry());
    }
}
