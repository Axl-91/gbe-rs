use super::*;

#[test]
fn load8_loads_value_into_all_registers() {
    let mut rng = rand::rng();

    let registers = [
        (0x06, Register8::B),
        (0x0E, Register8::C),
        (0x16, Register8::D),
        (0x1E, Register8::E),
        (0x26, Register8::H),
        (0x2E, Register8::L),
        (0x3E, Register8::A),
    ];

    for (opcode, register) in registers {
        let value: u8 = rng.random();

        let mut cpu = create_cpu(opcode, value);

        cpu.step();

        assert_eq!(cpu.get_register8(&register), value);
    }
}

#[test]
fn load8_advances_pc_by_two() {
    let mut rng = rand::rng();

    let opcode = 0x06;
    let value: u8 = rng.random();

    let mut cpu = create_cpu(opcode, value);

    cpu.step();

    assert_eq!(cpu.registers.get_pc(), GAME_ENTRY_POINT + 2);
}

#[test]
fn load8_preserves_flags() {
    let mut rng = rand::rng();

    let value: u8 = rng.random();

    let mut cpu = create_cpu(0x06, value);

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
