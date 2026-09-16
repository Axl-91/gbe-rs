use super::*;

const STOP_OPCODE: u8 = 0x10;

#[test]
fn stop_enters_stopped_state() {
    let mut rng = rand::rng();
    let second_byte: u8 = rng.random();

    let mut cpu = create_cpu(STOP_OPCODE, Some(second_byte), None);

    let t_cycles = cpu.step();

    assert_eq!(t_cycles, ControlInstruction::Stop.t_cycles());
    assert!(cpu.stopped);
}

#[test]
fn stop_consumes_second_byte() {
    let mut rng = rand::rng();
    let second_byte: u8 = rng.random();

    let mut cpu = create_cpu(STOP_OPCODE, Some(second_byte), None);

    cpu.step();

    assert_eq!(cpu.registers.get_pc(), GAME_ENTRY_POINT.wrapping_add(2));
}

#[test]
fn stopped_cpu_does_not_consume_cycles() {
    let mut rng = rand::rng();
    let second_byte: u8 = rng.random();

    let mut cpu = create_cpu(STOP_OPCODE, Some(second_byte), None);

    cpu.step();

    assert!(cpu.stopped);

    let t_cycles = cpu.step();

    assert_eq!(t_cycles, NON_CYCLES);
    assert!(cpu.stopped);
}
