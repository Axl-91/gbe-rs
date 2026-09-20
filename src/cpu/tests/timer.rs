use super::*;

const TIMER_DIV_ADDRESS: u16 = 0xFF04;
const TIMER_TIMA_ADDRESS: u16 = 0xFF05;
const TIMER_TAC_ADDRESS: u16 = 0xFF07;

#[test]
fn cpu_instructions_advance_timer_divider() {
    let mut cpu = create_cpu(0x00, None, None);
    cpu.bus.write(TIMER_DIV_ADDRESS, 0x00);

    for _ in 0..64 {
        cpu.step();
    }

    assert_eq!(cpu.bus.read(TIMER_DIV_ADDRESS), 1);
}

#[test]
fn cpu_instructions_increment_tima() {
    let mut cpu = create_cpu(0x00, None, None);

    cpu.bus.write(TIMER_DIV_ADDRESS, 0x00);
    cpu.bus.write(TIMER_TAC_ADDRESS, 0b101);

    for _ in 0..4 {
        cpu.step();
    }

    assert_eq!(cpu.bus.read(TIMER_DIV_ADDRESS), 0);
    assert_eq!(cpu.bus.read(TIMER_TIMA_ADDRESS), 1);
}

#[test]
fn writing_to_div_resets_divider() {
    let mut cpu = create_cpu(0x00, None, None);

    for _ in 0..64 {
        cpu.step();
    }

    assert_ne!(cpu.bus.read(TIMER_DIV_ADDRESS), 0);

    cpu.bus.write(TIMER_DIV_ADDRESS, 0xAB);

    assert_eq!(cpu.bus.read(TIMER_DIV_ADDRESS), 0);
}

#[test]
fn writing_to_div_can_increment_tima() {
    let mut cpu = create_cpu(0x00, None, None);

    cpu.bus.write(TIMER_DIV_ADDRESS, 0);
    cpu.bus.write(TIMER_TAC_ADDRESS, 0b101);

    for _ in 0..2 {
        cpu.step();
    }

    assert_eq!(cpu.bus.read(TIMER_TIMA_ADDRESS), 0);

    cpu.bus.write(TIMER_DIV_ADDRESS, 0);

    assert_eq!(cpu.bus.read(TIMER_TIMA_ADDRESS), 1);
}
