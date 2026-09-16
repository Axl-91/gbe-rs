use crate::memory::map::{INTERRUPT_ENABLE_ADDRESS, INTERRUPT_FLAG_ADDRESS};

use super::*;

const HALT_OPCODE: u8 = 0x76;
const INC_B_OPCODE: u8 = 0x04;
const INTERRUPT_MASK: u8 = 0x01;

#[test]
fn halt_enters_halted_state() {
    let mut cpu = create_cpu(HALT_OPCODE, None, None);

    cpu.step();

    assert!(cpu.halted);
}

#[test]
fn halt_remains_halted_without_pending_interrupt() {
    let mut cpu = create_cpu(HALT_OPCODE, None, None);

    cpu.step();

    assert!(cpu.halted);

    let t_cycles = cpu.step();

    assert!(cpu.halted);
    assert_eq!(t_cycles, 4);
}

#[test]
fn halt_wakes_when_interrupt_becomes_pending_with_ime_disabled() {
    let mut cpu = create_cpu(HALT_OPCODE, None, None);

    cpu.step();

    assert!(cpu.halted);

    cpu.bus.write(INTERRUPT_ENABLE_ADDRESS, INTERRUPT_MASK);
    cpu.bus.write(INTERRUPT_FLAG_ADDRESS, INTERRUPT_MASK);

    let t_cycles = cpu.step();

    assert!(!cpu.halted);
    assert_eq!(t_cycles, 0);
    assert!(!cpu.ime);
}

#[test]
fn halt_does_not_service_interrupt_when_ime_is_disabled() {
    let mut cpu = create_cpu(HALT_OPCODE, None, None);

    cpu.step();

    cpu.bus.write(INTERRUPT_ENABLE_ADDRESS, INTERRUPT_MASK);
    cpu.bus.write(INTERRUPT_FLAG_ADDRESS, INTERRUPT_MASK);

    cpu.step();

    assert!(!cpu.ime);
    assert_eq!(
        cpu.bus.read(INTERRUPT_FLAG_ADDRESS) & INTERRUPT_MASK,
        INTERRUPT_MASK
    );
}

#[test]
fn pending_interrupt_is_serviced_before_halt_when_ime_is_enabled() {
    let mut cpu = create_cpu(HALT_OPCODE, None, None);

    cpu.ime = true;

    let t_cycles = cpu.step();
    assert_eq!(t_cycles, ControlInstruction::Halt.t_cycles());

    assert!(cpu.halted);
    assert!(cpu.ime);

    cpu.bus.write(INTERRUPT_ENABLE_ADDRESS, INTERRUPT_MASK);
    cpu.bus.write(INTERRUPT_FLAG_ADDRESS, INTERRUPT_MASK);

    let t_cycles = cpu.step();

    assert!(!cpu.halted);
    assert!(!cpu.ime);
    assert!(t_cycles > 0);
}

#[test]
fn halt_bug_is_triggered_when_interrupt_is_already_pending() {
    let mut cpu = create_cpu(HALT_OPCODE, None, None);

    cpu.bus.write(INTERRUPT_ENABLE_ADDRESS, INTERRUPT_MASK);
    cpu.bus.write(INTERRUPT_FLAG_ADDRESS, INTERRUPT_MASK);

    cpu.step();

    assert!(!cpu.halted);
    assert!(cpu.halt_bug);
}

#[test]
fn halt_bug_does_not_increment_pc_on_next_fetch() {
    let mut cpu = create_cpu(HALT_OPCODE, None, None);

    cpu.bus.write(INTERRUPT_ENABLE_ADDRESS, INTERRUPT_MASK);
    cpu.bus.write(INTERRUPT_FLAG_ADDRESS, INTERRUPT_MASK);

    cpu.step();

    let pc_before_fetch = cpu.registers.get_pc();

    cpu.step();

    assert_eq!(cpu.registers.get_pc(), pc_before_fetch);
    assert!(!cpu.halt_bug);
}

#[test]
fn halt_bug_reads_next_opcode_twice() {
    let mut cpu = create_cpu(HALT_OPCODE, Some(INC_B_OPCODE), None);

    cpu.bus.write(INTERRUPT_ENABLE_ADDRESS, INTERRUPT_MASK);
    cpu.bus.write(INTERRUPT_FLAG_ADDRESS, INTERRUPT_MASK);

    cpu.step();

    let pc_after_halt = cpu.registers.get_pc();

    // This step shouldn't advance PC as we are in the halt_bug
    cpu.step();

    assert_eq!(cpu.registers.get_pc(), pc_after_halt);

    // Now we advance from PC as the halt bug is now taken care off
    cpu.step();

    assert_eq!(cpu.registers.get_pc(), pc_after_halt.wrapping_add(1));
}
