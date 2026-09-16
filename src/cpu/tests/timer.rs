use crate::memory::map::{TIMER_DIV_ADDRESS, TIMER_TAC_ADDRESS, TIMER_TIMA_ADDRESS};

use super::*;

#[test]
fn cpu_instructions_advance_timer_divider() {
    let mut cpu = create_cpu(0x00, None, None);

    for _ in 0..64 {
        let t_cycles = cpu.step();
        cpu.bus.tick(t_cycles);
    }

    assert_eq!(cpu.bus.read(TIMER_DIV_ADDRESS), 1);
}

#[test]
fn cpu_instructions_increment_tima() {
    let mut cpu = create_cpu(0x00, None, None);

    // Enable timer and select 262144 Hz.
    cpu.bus.write(TIMER_TAC_ADDRESS, 0b101);

    for _ in 0..4 {
        let t_cycles = cpu.step();
        cpu.bus.tick(t_cycles);
    }

    assert_eq!(cpu.bus.read(TIMER_DIV_ADDRESS), 0);
    assert_eq!(cpu.bus.read(TIMER_TIMA_ADDRESS), 1);
}

#[test]
fn writing_to_div_resets_divider() {
    let mut cpu = create_cpu(0x00, None, None);

    for _ in 0..64 {
        let t_cycles = cpu.step();
        cpu.bus.tick(t_cycles);
    }

    assert_ne!(cpu.bus.read(TIMER_DIV_ADDRESS), 0);

    cpu.bus.write(TIMER_DIV_ADDRESS, 0xAB);

    assert_eq!(cpu.bus.read(TIMER_DIV_ADDRESS), 0);
}

#[test]
fn writing_to_div_can_increment_tima() {
    let mut cpu = create_cpu(0x00, None, None);

    // Enable timer and select DIV bit 3 (262144 Hz).
    cpu.bus.write(TIMER_TAC_ADDRESS, 0b101);

    // Advance until DIV bit 3 is high.
    for _ in 0..2 {
        let t_cycles = cpu.step();
        cpu.bus.tick(t_cycles);
    }

    assert_eq!(cpu.bus.read(TIMER_TIMA_ADDRESS), 0);

    // Resetting DIV changes the selected bit from 1 to 0,
    // producing a falling edge and incrementing TIMA.
    cpu.bus.write(TIMER_DIV_ADDRESS, 0);

    assert_eq!(cpu.bus.read(TIMER_TIMA_ADDRESS), 1);
}
