use super::Timer;
use rand::RngExt;

#[test]
fn timer_starts_with_zeroed_registers() {
    let timer = Timer::new();

    assert_eq!(timer.read_div(), 0);
    assert_eq!(timer.read_tima(), 0);
    assert_eq!(timer.read_tma(), 0);
    // The 3 upper bits of TAC are unused and always read back as 1.
    assert_eq!(timer.read_tac(), 0xF8);
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

    let _ = timer.write_div();

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
    // Only the lower 3 bits of TAC are actually stored.
    let value: u8 = rng.random_range(0..=0b111);

    let mut timer = Timer::new();

    let _ = timer.write_tac(value);

    assert_eq!(timer.read_tac(), value | 0xF8);
}

#[test]
fn frequency_selects_correct_divider_bit() {
    let mut timer = Timer::new();

    let _ = timer.write_tac(0b00);
    assert_eq!(timer.get_frequency_bit(), 9);

    let _ = timer.write_tac(0b01);
    assert_eq!(timer.get_frequency_bit(), 3);

    let _ = timer.write_tac(0b10);
    assert_eq!(timer.get_frequency_bit(), 5);

    let _ = timer.write_tac(0b11);
    assert_eq!(timer.get_frequency_bit(), 7);
}

#[test]
fn tima_does_not_increment_when_timer_is_disabled() {
    let mut rng = rand::rng();
    let initial_tima: u8 = rng.random();

    let mut timer = Timer::new();

    timer.write_tima(initial_tima);
    let _ = timer.write_tac(0);

    for _ in 0..1024 {
        timer.tick();
    }

    assert_eq!(timer.read_tima(), initial_tima);
}

#[test]
fn tima_increments_on_falling_edge() {
    let mut timer = Timer::new();

    // Timer enabled, frequency = 4096 Hz (bit 9).
    let _ = timer.write_tac(0b100);

    for _ in 0..1024 {
        timer.tick();
    }

    assert_eq!(timer.read_tima(), 1);
}

#[test]
fn tima_reads_zero_during_the_m_cycle_right_after_overflow() {
    let mut timer = Timer::new();

    let _ = timer.write_tac(0b100);
    timer.write_tima(0xFF);

    // The tick where the falling edge overflows TIMA.
    for _ in 0..1024 {
        timer.tick();
    }

    // TIMA reads 0x00 for the whole M-Cycle right after the overflow;
    // TMA hasn't been copied in yet.
    assert_eq!(timer.read_tima(), 0x00);
}

#[test]
fn tima_overflow_reloads_from_tma_one_m_cycle_later() {
    let mut rng = rand::rng();
    let modulo: u8 = rng.random();

    let mut timer = Timer::new();

    let _ = timer.write_tac(0b100);
    timer.write_tma(modulo);
    timer.write_tima(0xFF);

    // 1024 ticks to overflow, plus one M-Cycle (4 T-Cycles) for the delayed reload.
    for _ in 0..1028 {
        timer.tick();
    }

    assert_eq!(timer.read_tima(), modulo);
}

#[test]
fn overflow_event_is_emitted_exactly_once_one_m_cycle_after_overflow() {
    let mut timer = Timer::new();

    let _ = timer.write_tac(0b100);
    timer.write_tima(0xFF);

    let mut events = 0;

    for _ in 0..1028 {
        if timer.tick().is_some() {
            events += 1;
        }
    }

    assert_eq!(events, 1);
}

#[test]
fn writing_tima_during_the_overflow_m_cycle_cancels_the_reload() {
    let mut rng = rand::rng();
    let modulo: u8 = rng.random();
    let written: u8 = rng.random();

    let mut timer = Timer::new();

    let _ = timer.write_tac(0b100);
    timer.write_tma(modulo);
    timer.write_tima(0xFF);

    // Advance to the overflow tick (TIMA == 0x00, reload pending).
    for _ in 0..1024 {
        timer.tick();
    }

    // A write during this M-Cycle cancels the pending reload entirely:
    // TMA is never copied in, and no interrupt is requested.
    timer.write_tima(written);

    let mut events = 0;
    for _ in 0..8 {
        if timer.tick().is_some() {
            events += 1;
        }
    }

    assert_eq!(timer.read_tima(), written);
    assert_eq!(events, 0);
}

#[test]
fn writing_tima_during_the_reload_m_cycle_is_ignored() {
    let mut rng = rand::rng();
    let modulo: u8 = rng.random();

    let mut timer = Timer::new();

    let _ = timer.write_tac(0b100);
    timer.write_tma(modulo);
    timer.write_tima(0xFF);

    // Advance to the tick where TMA gets copied into TIMA.
    for _ in 0..1028 {
        timer.tick();
    }

    assert_eq!(timer.read_tima(), modulo);

    // A write during this same M-Cycle is overwritten by the reload again.
    timer.write_tima(0x00);

    assert_eq!(timer.read_tima(), modulo);
}

#[test]
fn writing_tma_during_the_reload_m_cycle_is_also_copied_to_tima() {
    let mut rng = rand::rng();
    let old_modulo: u8 = rng.random();
    let new_modulo: u8 = rng.random();

    let mut timer = Timer::new();

    let _ = timer.write_tac(0b100);
    timer.write_tma(old_modulo);
    timer.write_tima(0xFF);

    for _ in 0..1028 {
        timer.tick();
    }

    assert_eq!(timer.read_tima(), old_modulo);

    // A write to TMA during the reload M-Cycle lands in TIMA as well.
    timer.write_tma(new_modulo);

    assert_eq!(timer.read_tima(), new_modulo);
}

#[test]
fn writing_div_can_trigger_an_early_tick_when_selected_bit_is_set() {
    let mut timer = Timer::new();

    // Timer enabled, frequency = 4096 Hz (bit 9).
    let _ = timer.write_tac(0b100);

    // Advance div until bit 9 is set (halfway through its period).
    for _ in 0..512 {
        timer.tick();
    }

    let _ = timer.write_div();

    // Resetting the counter dropped the selected bit from 1 to 0,
    // producing an early tick.
    assert_eq!(timer.read_tima(), 1);
    assert_eq!(timer.read_div(), 0);
}

#[test]
fn writing_tac_can_trigger_an_early_tick_when_disabling_with_selected_bit_set() {
    let mut timer = Timer::new();

    // Timer enabled, frequency = 4096 Hz (bit 9).
    let _ = timer.write_tac(0b100);

    // Advance div until bit 9 is set.
    for _ in 0..512 {
        timer.tick();
    }

    // Disabling the timer while the selected bit is set ticks once (DMG).
    let _ = timer.write_tac(0b000);

    assert_eq!(timer.read_tima(), 1);
}

#[test]
fn writing_tac_does_not_tick_when_switching_to_a_bit_that_is_already_unset() {
    let mut timer = Timer::new();

    // Timer enabled, frequency = 4096 Hz (bit 9), div stays at 0 so bit 9 is unset.
    let _ = timer.write_tac(0b100);

    // Switch to frequency 262144 Hz (bit 3), also unset at div == 0.
    let _ = timer.write_tac(0b101);

    assert_eq!(timer.read_tima(), 0);
}
