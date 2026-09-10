use super::*;

#[test]
fn load8_register_copies_value() {
    let mut rng = rand::rng();

    let instructions = [
        (0x40, Register8::B, Register8::B),
        (0x41, Register8::B, Register8::C),
        (0x42, Register8::B, Register8::D),
        (0x43, Register8::B, Register8::E),
        (0x44, Register8::B, Register8::H),
        (0x45, Register8::B, Register8::L),
        (0x47, Register8::B, Register8::A),
        (0x48, Register8::C, Register8::B),
        (0x49, Register8::C, Register8::C),
        (0x4A, Register8::C, Register8::D),
        (0x4B, Register8::C, Register8::E),
        (0x4C, Register8::C, Register8::H),
        (0x4D, Register8::C, Register8::L),
        (0x4F, Register8::C, Register8::A),
        (0x50, Register8::D, Register8::B),
        (0x51, Register8::D, Register8::C),
        (0x52, Register8::D, Register8::D),
        (0x53, Register8::D, Register8::E),
        (0x54, Register8::D, Register8::H),
        (0x55, Register8::D, Register8::L),
        (0x57, Register8::D, Register8::A),
        (0x58, Register8::E, Register8::B),
        (0x59, Register8::E, Register8::C),
        (0x5A, Register8::E, Register8::D),
        (0x5B, Register8::E, Register8::E),
        (0x5C, Register8::E, Register8::H),
        (0x5D, Register8::E, Register8::L),
        (0x5F, Register8::E, Register8::A),
        (0x60, Register8::H, Register8::B),
        (0x61, Register8::H, Register8::C),
        (0x62, Register8::H, Register8::D),
        (0x63, Register8::H, Register8::E),
        (0x64, Register8::H, Register8::H),
        (0x65, Register8::H, Register8::L),
        (0x67, Register8::H, Register8::A),
        (0x68, Register8::L, Register8::B),
        (0x69, Register8::L, Register8::C),
        (0x6A, Register8::L, Register8::D),
        (0x6B, Register8::L, Register8::E),
        (0x6C, Register8::L, Register8::H),
        (0x6D, Register8::L, Register8::L),
        (0x6F, Register8::L, Register8::A),
        (0x78, Register8::A, Register8::B),
        (0x79, Register8::A, Register8::C),
        (0x7A, Register8::A, Register8::D),
        (0x7B, Register8::A, Register8::E),
        (0x7C, Register8::A, Register8::H),
        (0x7D, Register8::A, Register8::L),
        (0x7F, Register8::A, Register8::A),
    ];

    for (opcode, destination, source) in instructions {
        let value: u8 = rng.random();

        let mut cpu = create_cpu(opcode, 0);

        cpu.set_register8(&source, value);

        cpu.step();

        assert_eq!(cpu.get_register8(&destination), value);
    }
}

#[test]
fn load8_register_advances_pc_by_one() {
    let mut rng = rand::rng();
    let value: u8 = rng.random();

    let mut cpu = create_cpu(0x41, 0);

    cpu.set_register8(&Register8::C, value);

    cpu.step();

    assert_eq!(cpu.registers.get_pc(), GAME_ENTRY_POINT + 1);
}

#[test]
fn load8_register_preserves_flags() {
    let mut rng = rand::rng();
    let value: u8 = rng.random();

    let mut cpu = create_cpu(0x41, 0);

    cpu.set_register8(&Register8::C, value);

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
