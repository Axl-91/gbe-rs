use super::Timer;
use rand::RngExt;

#[test]
fn timer_starts_with_zeroed_registers() {
    let timer = Timer::new();

    assert_eq!(timer.read_div(), 0);
    assert_eq!(timer.read_tima(), 0);
    assert_eq!(timer.read_tma(), 0);
    assert_eq!(timer.read_tac(), 0);
}

#[test]
fn div_increments_every_t_cycle() {
    let mut timer = Timer::new();

    for _ in 0..256 {
        timer.tick();
    }

    assert_eq!(timer.read_div(), 1);
}

#[test]
fn resetting_div_clears_divider() {
    let mut timer = Timer::new();

    for _ in 0..256 {
        timer.tick();
    }

    timer.reset_div();

    assert_eq!(timer.read_div(), 0);
}

#[test]
fn tima_can_be_written_and_read() {
    let mut rng = rand::rng();
    let value: u8 = rng.random();

    let mut timer = Timer::new();

    timer.write_tima(value);

    assert_eq!(timer.read_tima(), value);
}

#[test]
fn tma_can_be_written_and_read() {
    let mut rng = rand::rng();
    let value: u8 = rng.random();

    let mut timer = Timer::new();

    timer.write_tma(value);

    assert_eq!(timer.read_tma(), value);
}

#[test]
fn tac_can_be_written_and_read() {
    let mut rng = rand::rng();
    let value: u8 = rng.random();

    let mut timer = Timer::new();

    timer.write_tac(value);

    assert_eq!(timer.read_tac(), value);
}

#[test]
fn frequency_selects_correct_divider_bit() {
    let mut timer = Timer::new();

    timer.write_tac(0b00);
    assert_eq!(timer.get_frequency_bit(), 9);

    timer.write_tac(0b01);
    assert_eq!(timer.get_frequency_bit(), 3);

    timer.write_tac(0b10);
    assert_eq!(timer.get_frequency_bit(), 5);

    timer.write_tac(0b11);
    assert_eq!(timer.get_frequency_bit(), 7);
}

#[test]
fn tima_does_not_increment_when_timer_is_disabled() {
    let mut rng = rand::rng();
    let initial_tima: u8 = rng.random();

    let mut timer = Timer::new();

    timer.write_tima(initial_tima);
    timer.write_tac(0);

    for _ in 0..1024 {
        timer.tick();
    }

    assert_eq!(timer.read_tima(), initial_tima);
}

#[test]
fn tima_increments_on_falling_edge() {
    let mut timer = Timer::new();

    // Timer enabled, frequency = 4096 Hz.
    timer.write_tac(0b100);

    for _ in 0..1024 {
        timer.tick();
    }

    assert_eq!(timer.read_tima(), 1);
}

#[test]
fn tima_overflow_reloads_from_tma() {
    let mut rng = rand::rng();
    let modulo: u8 = rng.random();

    let mut timer = Timer::new();

    // Timer enabled, frequency = 4096 Hz.
    timer.write_tac(0b100);

    timer.write_tma(modulo);
    timer.write_tima(0xFF);

    for _ in 0..1024 {
        timer.tick();
    }

    assert_eq!(timer.read_tima(), modulo);
}
