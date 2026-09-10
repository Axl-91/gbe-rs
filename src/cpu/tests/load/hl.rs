use super::*;

#[test]
fn load8_from_hl_copies_value_to_all_registers() {
    let mut rng = rand::rng();

    let instructions = [
        (0x46, Register8::B),
        (0x4E, Register8::C),
        (0x56, Register8::D),
        (0x5E, Register8::E),
        (0x66, Register8::H),
        (0x6E, Register8::L),
        (0x7E, Register8::A),
    ];

    for (opcode, destination) in instructions {
        let value: u8 = rng.random();

        let mut cpu = create_cpu(opcode, None, None);
        let address = get_rand_wram_address();

        cpu.registers.set_hl(address);
        cpu.bus.write(address, value);

        cpu.step();

        assert_eq!(cpu.get_register8(&destination), value);
    }
}

#[test]
fn load8_from_hl_advances_pc_by_one() {
    let mut rng = rand::rng();
    let value: u8 = rng.random();

    let mut cpu = create_cpu(0x46, None, None);
    let address = get_rand_wram_address();

    cpu.registers.set_hl(address);
    cpu.bus.write(address, value);

    cpu.step();

    assert_eq!(cpu.registers.get_pc(), GAME_ENTRY_POINT + 1);
}

#[test]
fn load8_from_hl_preserves_flags() {
    let mut rng = rand::rng();
    let value: u8 = rng.random();

    let mut cpu = create_cpu(0x46, None, None);
    let address = get_rand_wram_address();

    cpu.registers.set_hl(address);
    cpu.bus.write(address, value);

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
fn load8_to_hl_copies_value_from_all_registers() {
    let mut rng = rand::rng();

    let instructions = [
        (0x70, Register8::B),
        (0x71, Register8::C),
        (0x72, Register8::D),
        (0x73, Register8::E),
        (0x74, Register8::H),
        (0x75, Register8::L),
        (0x77, Register8::A),
    ];

    for (opcode, source) in instructions {
        let value: u8 = rng.random();

        let mut cpu = create_cpu(opcode, None, None);
        let address = get_rand_wram_address();

        cpu.registers.set_hl(address);

        match source {
            // H and L are part of HL, so their values
            // cannot be changed without changing the address.
            Register8::H | Register8::L => {}
            _ => {
                cpu.set_register8(&source, value);
            }
        }

        cpu.step();

        let expected = match source {
            Register8::H => cpu.registers.get_h(),
            Register8::L => cpu.registers.get_l(),
            _ => value,
        };

        assert_eq!(cpu.bus.read(address), expected);
    }
}

#[test]
fn load8_to_hl_advances_pc_by_one() {
    let mut rng = rand::rng();
    let value: u8 = rng.random();

    let mut cpu = create_cpu(0x70, None, None);
    let address = get_rand_wram_address();

    cpu.set_register8(&Register8::B, value);
    cpu.registers.set_hl(address);

    cpu.step();

    assert_eq!(cpu.registers.get_pc(), GAME_ENTRY_POINT + 1);
}

#[test]
fn load8_to_hl_preserves_flags() {
    let mut rng = rand::rng();
    let value: u8 = rng.random();

    let mut cpu = create_cpu(0x70, None, None);

    cpu.set_register8(&Register8::B, value);
    let address = get_rand_wram_address();
    cpu.registers.set_hl(address);

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
fn load8_to_hl_immediate_writes_value_to_memory() {
    let mut rng = rand::rng();

    let value: u8 = rng.random();
    let address = get_rand_wram_address();

    let mut cpu = create_cpu(0x36, Some(value), None);

    cpu.registers.set_hl(address);

    cpu.step();

    assert_eq!(cpu.bus.read(address), value);
}

#[test]
fn load8_to_hl_immediate_advances_pc_by_two() {
    let mut rng = rand::rng();

    let value: u8 = rng.random();

    let mut cpu = create_cpu(0x36, Some(value), None);
    let address = get_rand_wram_address();

    cpu.registers.set_hl(address);

    cpu.step();

    assert_eq!(cpu.registers.get_pc(), GAME_ENTRY_POINT + 2);
}

#[test]
fn load8_to_hl_immediate_preserves_flags() {
    let mut rng = rand::rng();

    let value: u8 = rng.random();

    let mut cpu = create_cpu(0x36, Some(value), None);
    let address = get_rand_wram_address();

    cpu.registers.set_hl(address);

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
