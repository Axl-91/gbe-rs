use super::*;

/// Ticks the timer and returns how many interrupts were requested.
fn tick_n(timer: &mut Timer, t_cycles: u32) -> u32 {
    let mut interrupts = 0;
    for _ in 0..t_cycles {
        if timer.tick() {
            interrupts += 1;
        }
    }
    interrupts
}

/// Timer configured the way Blargg's `init_timer` does: TMA = 0, TAC = 5 (262144 Hz).
fn configured_timer() -> Timer {
    let mut timer = Timer::new();
    timer.write(TMA_ADDRESS, 0);
    timer.write(TAC_ADDRESS, 0x05);
    timer
}

/// Timer whose TIMA has just overflowed: TIMA reads 0x00 and the reload is pending.
fn just_overflowed_timer(tma: u8) -> Timer {
    let mut timer = configured_timer();
    timer.write(TMA_ADDRESS, tma);
    timer.write(DIV_ADDRESS, 0);
    timer.write(TIMA_ADDRESS, 0xFF);
    assert_eq!(tick_n(&mut timer, 16), 0);
    assert_eq!(timer.read(TIMA_ADDRESS), 0x00);
    timer
}

// ---------------------------------------------------------------
// DIV
// ---------------------------------------------------------------

#[test]
fn div_increments_every_256_t_cycles() {
    let mut timer = Timer::new();
    timer.write(DIV_ADDRESS, 0);

    tick_n(&mut timer, 255);
    assert_eq!(timer.read(DIV_ADDRESS), 0);

    tick_n(&mut timer, 1);
    assert_eq!(timer.read(DIV_ADDRESS), 1);

    tick_n(&mut timer, 256 * 3);
    assert_eq!(timer.read(DIV_ADDRESS), 4);
}

#[test]
fn writing_div_resets_the_internal_counter() {
    let mut timer = Timer::new();
    tick_n(&mut timer, 1234);

    timer.write(DIV_ADDRESS, 0x55);

    assert_eq!(timer.sys_counter, 0);
    assert_eq!(timer.read(DIV_ADDRESS), 0);
}

// ---------------------------------------------------------------
// TIMA frequency and enable
// ---------------------------------------------------------------

#[test]
fn tima_increments_every_16_t_cycles_at_262144_hz() {
    let mut timer = configured_timer();
    timer.write(DIV_ADDRESS, 0);
    timer.write(TIMA_ADDRESS, 0);

    tick_n(&mut timer, 15);
    assert_eq!(timer.read(TIMA_ADDRESS), 0);

    tick_n(&mut timer, 1);
    assert_eq!(timer.read(TIMA_ADDRESS), 1);

    tick_n(&mut timer, 16 * 9);
    assert_eq!(timer.read(TIMA_ADDRESS), 10);
}

#[test]
fn tima_period_matches_each_tac_clock_select() {
    let cases = [(0b100u8, 1024u32), (0b101, 16), (0b110, 64), (0b111, 256)];

    for (tac, period) in cases {
        let mut timer = Timer::new();
        timer.write(TAC_ADDRESS, tac);
        timer.write(DIV_ADDRESS, 0);
        timer.write(TIMA_ADDRESS, 0);

        tick_n(&mut timer, period * 10);

        assert_eq!(timer.read(TIMA_ADDRESS), 10, "TAC = {tac:#05b}");
    }
}

#[test]
fn tima_does_not_increment_when_timer_is_disabled() {
    let mut timer = Timer::new();
    timer.write(TAC_ADDRESS, 0b001); // 262144 Hz but enable bit clear
    timer.write(TIMA_ADDRESS, 0);

    tick_n(&mut timer, 10_000);

    assert_eq!(timer.read(TIMA_ADDRESS), 0);
}

#[test]
fn tac_unused_bits_read_as_one() {
    let mut timer = Timer::new();
    timer.write(TAC_ADDRESS, 0x05);

    assert_eq!(timer.read(TAC_ADDRESS), 0xFD);
}

// ---------------------------------------------------------------
// Falling-edge side effects
// ---------------------------------------------------------------

#[test]
fn writing_tima_does_not_reset_the_counter_phase() {
    let mut timer = configured_timer();
    tick_n(&mut timer, 7);
    let counter_before = timer.sys_counter;

    timer.write(TIMA_ADDRESS, 0);

    assert_eq!(timer.sys_counter, counter_before);
}

#[test]
fn writing_div_while_selected_bit_is_high_increments_tima() {
    let mut timer = configured_timer();
    timer.write(DIV_ADDRESS, 0);
    timer.write(TIMA_ADDRESS, 0);
    tick_n(&mut timer, 8); // counter = 8, bit 3 is now high

    timer.write(DIV_ADDRESS, 0);

    assert_eq!(timer.read(TIMA_ADDRESS), 1);
}

#[test]
fn writing_div_while_selected_bit_is_low_does_not_increment_tima() {
    let mut timer = configured_timer();
    timer.write(DIV_ADDRESS, 0);
    timer.write(TIMA_ADDRESS, 0);
    tick_n(&mut timer, 4); // counter = 4, bit 3 still low

    timer.write(DIV_ADDRESS, 0);

    assert_eq!(timer.read(TIMA_ADDRESS), 0);
}

#[test]
fn disabling_timer_while_selected_bit_is_high_increments_tima() {
    let mut timer = configured_timer();
    timer.write(DIV_ADDRESS, 0);
    timer.write(TIMA_ADDRESS, 0);
    tick_n(&mut timer, 8); // bit 3 is high

    timer.write(TAC_ADDRESS, 0b001); // clears the enable bit

    assert_eq!(timer.read(TIMA_ADDRESS), 1);
}

#[test]
fn enabling_timer_never_increments_tima() {
    let mut timer = Timer::new();
    timer.write(DIV_ADDRESS, 0);
    tick_n(&mut timer, 8); // bit 3 is high while the timer is disabled
    timer.write(TIMA_ADDRESS, 0);

    timer.write(TAC_ADDRESS, 0b101);

    assert_eq!(timer.read(TIMA_ADDRESS), 0);
}

// ---------------------------------------------------------------
// Overflow, reload and interrupt
// ---------------------------------------------------------------

#[test]
fn overflow_reads_zero_then_loads_tma_and_requests_interrupt_after_4_t_cycles() {
    let mut timer = just_overflowed_timer(0x23);

    for _ in 0..3 {
        assert!(!timer.tick());
        assert_eq!(timer.read(TIMA_ADDRESS), 0x00);
    }

    assert!(timer.tick());
    assert_eq!(timer.read(TIMA_ADDRESS), 0x23);
}

#[test]
fn writing_tima_during_overflow_delay_cancels_reload_and_interrupt() {
    let mut timer = just_overflowed_timer(0x23);

    timer.write(TIMA_ADDRESS, 0x55);
    let interrupts = tick_n(&mut timer, 10);

    assert_eq!(interrupts, 0);
    assert_eq!(timer.read(TIMA_ADDRESS), 0x55);
}

#[test]
fn writing_tima_in_the_reload_window_is_ignored() {
    let mut timer = just_overflowed_timer(0x23);
    tick_n(&mut timer, 4); // reload happens on the 4th tick

    timer.write(TIMA_ADDRESS, 0x99);

    assert_eq!(timer.read(TIMA_ADDRESS), 0x23);
}

#[test]
fn writing_tima_after_the_reload_window_is_accepted() {
    let mut timer = just_overflowed_timer(0x23);
    tick_n(&mut timer, 4 + 4);

    timer.write(TIMA_ADDRESS, 0x99);

    assert_eq!(timer.read(TIMA_ADDRESS), 0x99);
}

#[test]
fn writing_tma_in_the_reload_window_also_updates_tima() {
    let mut timer = just_overflowed_timer(0x23);
    tick_n(&mut timer, 4);

    timer.write(TMA_ADDRESS, 0x77);

    assert_eq!(timer.read(TIMA_ADDRESS), 0x77);
}

#[test]
fn counting_resumes_from_tma_after_overflow() {
    let mut timer = just_overflowed_timer(0xF0);
    tick_n(&mut timer, 4); // reload: TIMA = 0xF0

    // Two more falling edges (counter 32 and 48).
    tick_n(&mut timer, 32);

    assert_eq!(timer.read(TIMA_ADDRESS), 0xF2);
}

// ---------------------------------------------------------------
// Blargg's cycle-accurate timer routines (timer.s)
//
// The harness below models the M-cycles spent by `init_timer`,
// `start_timer`, `stop_timer` and `stop_timer_word`, running the timer
// alongside them. If these pass, the timer is not what makes
// `instr_timing` fail: the CPU instruction cycle counts are.
// ---------------------------------------------------------------

struct Harness {
    timer: Timer,
    interrupt_flag: bool,
}

impl Harness {
    /// `phase_t_cycles` offsets the start so every counter alignment is exercised.
    fn new(phase_t_cycles: u32) -> Self {
        let mut harness = Self {
            timer: Timer::new(),
            interrupt_flag: false,
        };
        harness.timer.write(TMA_ADDRESS, 0);
        harness.timer.write(TAC_ADDRESS, 0x05);
        harness.tick_t_cycles(phase_t_cycles);
        harness
    }

    fn tick_t_cycles(&mut self, t_cycles: u32) {
        for _ in 0..t_cycles {
            if self.timer.tick() {
                self.interrupt_flag = true;
            }
        }
    }

    fn wait(&mut self, m_cycles: u32) {
        self.tick_t_cycles(m_cycles * 4);
    }

    /// `ldh (n),a`: 3 M-cycles, the bus write happens on the last one.
    fn ldh_write(&mut self, address: u16, value: u8) {
        self.wait(2);
        self.timer.write(address, value);
        self.wait(1);
    }

    /// `ldh a,(n)`: 3 M-cycles, the bus read happens on the last one.
    fn ldh_read(&mut self, address: u16) -> u8 {
        self.wait(2);
        let value = self.timer.read(address);
        self.wait(1);
        value
    }

    /// `ldh a,(IF)` followed by looking at bit 2.
    fn read_timer_interrupt_flag(&mut self) -> bool {
        self.wait(2);
        let value = self.interrupt_flag;
        self.wait(1);
        value
    }

    /// `call start_timer`
    fn start_timer(&mut self) {
        self.wait(6); // call
        self.wait(4); // push af
        loop {
            self.wait(1); // xor a
            self.ldh_write(TIMA_ADDRESS, 0);
            let tima = self.ldh_read(TIMA_ADDRESS);
            self.wait(1); // or a
            if tima == 0 {
                self.wait(2); // jr nz, not taken
                break;
            }
            self.wait(3); // jr nz, taken
        }
        self.wait(3); // pop af
        self.wait(4); // ret
    }

    /// Body of `stop_timer_word` (the `call` is charged by the caller). Returns DE.
    fn stop_timer_word(&mut self) -> u16 {
        self.wait(2); // ld d,0
        let tima = self.ldh_read(TIMA_ADDRESS);
        self.wait(2); // sub 5
        let a = tima.wrapping_sub(5);
        self.wait(1 + 2 + 1 + 2 + 1); // add a, rl d, add a, rl d, ld e,a
        let mut de = (a as u16) << 2;

        loop {
            self.wait(1); // xor a
            self.ldh_write(TIMA_ADDRESS, 0);
            let tima = self.ldh_read(TIMA_ADDRESS);
            de = de.wrapping_sub(1);
            self.wait(2); // dec de
            self.wait(1); // or a
            if tima == 0 {
                self.wait(2); // jr nz, not taken
                break;
            }
            self.wait(3); // jr nz, taken
        }

        self.wait(4); // ret
        de
    }

    /// `call stop_timer`. Returns A.
    fn stop_timer(&mut self) -> u8 {
        self.wait(6); // call stop_timer
        self.wait(4); // push de
        self.wait(6); // call stop_timer_word
        let de = self.stop_timer_word();
        self.wait(1); // ld a,e
        self.wait(2); // sub 10
        self.wait(3); // pop de
        self.wait(4); // ret
        (de as u8).wrapping_sub(10)
    }

    /// start_timer, `delay_m_cycles` of other code, stop_timer.
    fn measure(&mut self, delay_m_cycles: u32) -> u8 {
        self.start_timer();
        self.wait(delay_m_cycles);
        self.stop_timer()
    }
}

#[test]
fn blargg_test_timer_measures_zero_cycles_for_every_phase() {
    for phase in 0..16 {
        let mut harness = Harness::new(phase);

        assert_eq!(harness.measure(0), 0, "phase {phase}");
    }
}

#[test]
fn blargg_stop_timer_returns_elapsed_m_cycles_for_every_phase() {
    for phase in 0..16 {
        for delay in 0..=200u32 {
            let mut harness = Harness::new(phase);

            assert_eq!(
                harness.measure(delay),
                delay as u8,
                "phase {phase}, delay {delay}"
            );
        }
    }
}

#[test]
fn blargg_init_timer_overflow_window_holds_for_every_phase() {
    for phase in 0..16 {
        let mut harness = Harness::new(phase);

        harness.interrupt_flag = false; // wreg IF,0
        harness.ldh_write(TIMA_ADDRESS, 0xEC); // wreg TIMA,-20
        harness.wait(70); // delay 70

        let too_early = harness.read_timer_interrupt_flag();
        harness.wait(2); // and $04
        harness.wait(3); // jp nz, not taken
        let late_enough = harness.read_timer_interrupt_flag();

        assert!(!too_early, "TIMA expired too early, phase {phase}");
        assert!(late_enough, "TIMA took too long to expire, phase {phase}");
    }
}
