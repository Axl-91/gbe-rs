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
