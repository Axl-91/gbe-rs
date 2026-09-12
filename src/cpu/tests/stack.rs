use super::*;

#[test]
fn push_register16() {
    let mut rng = rand::rng();

    let opcodes = [
        0xC5, // PUSH BC
        0xD5, // PUSH DE
        0xE5, // PUSH HL
        0xF5, // PUSH AF
    ];

    for opcode in opcodes {
        let mut cpu = create_cpu(opcode, None, None);

        let value = rng.random();
        let sp = get_rand_wram_address();

        match opcode {
            0xC5 => cpu.registers.set_bc(value),
            0xD5 => cpu.registers.set_de(value),
            0xE5 => cpu.registers.set_hl(value),
            0xF5 => cpu.registers.set_af(value),
            _ => unreachable!(),
        }

        cpu.registers.set_sp(sp);

        cpu.step();

        let [lower_value, higher_value] = value.to_le_bytes();

        assert_eq!(cpu.bus.read(sp - 1), higher_value);
        assert_eq!(cpu.bus.read(sp - 2), lower_value);
        assert_eq!(cpu.registers.get_sp(), sp - 2);
    }
}

#[test]
fn push_register16_advances_pc_by_one() {
    let opcodes = [
        0xC5, // PUSH BC
        0xD5, // PUSH DE
        0xE5, // PUSH HL
        0xF5, // PUSH AF
    ];

    for opcode in opcodes {
        let mut cpu = create_cpu(opcode, None, None);

        cpu.registers.set_sp(get_rand_wram_address());

        cpu.step();

        assert_eq!(cpu.registers.get_pc(), GAME_ENTRY_POINT + 1);
    }
}

#[test]
fn pop_register16() {
    let mut rng = rand::rng();

    let opcodes = [
        0xC1, // POP BC
        0xD1, // POP DE
        0xE1, // POP HL
        0xF1, // POP AF
    ];

    for opcode in opcodes {
        let mut cpu = create_cpu(opcode, None, None);

        let mut value: u16 = rng.random();
        let sp = get_rand_wram_address();

        let [lower_value, higher_value] = value.to_le_bytes();

        cpu.registers.set_sp(sp);

        // Value -> Memory[SP]
        cpu.bus.write(sp, lower_value);

        // Value -> Memory[SP + 1]
        cpu.bus.write(sp + 1, higher_value);

        cpu.step();

        let actual_value = match opcode {
            0xC1 => cpu.registers.get_bc(),
            0xD1 => cpu.registers.get_de(),
            0xE1 => cpu.registers.get_hl(),
            0xF1 => cpu.registers.get_af(),
            _ => unreachable!(),
        };

        // The lower 4 bits of register F are always zero
        if opcode == 0xF1 {
            value &= 0xFFF0;
        }

        assert_eq!(actual_value, value);
        assert_eq!(cpu.registers.get_sp(), sp + 2);
    }
}

#[test]
fn pop_register16_advances_pc_by_one() {
    let opcodes = [
        0xC1, // POP BC
        0xD1, // POP DE
        0xE1, // POP HL
        0xF1, // POP AF
    ];

    for opcode in opcodes {
        let mut cpu = create_cpu(opcode, None, None);

        cpu.registers.set_sp(get_rand_wram_address());

        cpu.step();

        assert_eq!(cpu.registers.get_pc(), GAME_ENTRY_POINT + 1);
    }
}

#[test]
fn pop_af_clears_unused_flag_bits() {
    let mut cpu = create_cpu(0xF1, None, None);

    let sp = WRAM_START;

    cpu.registers.set_sp(sp);

    cpu.bus.write(sp, u8::MAX);
    cpu.bus.write(sp + 1, u8::MAX);

    cpu.step();

    assert_eq!(cpu.registers.get_af() & 0x0F, 0);
}
