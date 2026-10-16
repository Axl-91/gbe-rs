#![allow(dead_code)]

use crate::apu::envelope;
use crate::apu::length_counter::LengthCounter;

const NR41_ADDRESS: u16 = 0xFF20;
const NR42_ADDRESS: u16 = 0xFF21;
const NR43_ADDRESS: u16 = 0xFF22;
const NR44_ADDRESS: u16 = 0xFF23;

const NR41_LENGTH_MASK: u8 = 0b0011_1111;

const NR44_TRIGGER_MASK: u8 = 0b1000_0000;
const NR44_LENGTH_ENABLE_MASK: u8 = 0b0100_0000;

const NR41_READ_VALUE: u8 = 0xFF;
const NR42_READ_MASK: u8 = 0b0000_0000;
const NR43_READ_MASK: u8 = 0b0000_0000;
const NR44_READ_MASK: u8 = 0b1011_1111;

const NR43_CLOCK_SHIFT_MASK: u8 = 0b1111_0000;
const NR43_WIDTH_MODE_MASK: u8 = 0b0000_1000;
const NR43_DIVISOR_MASK: u8 = 0b0000_0111;

const LFSR_INITIAL_VALUE: u16 = 0x7FFF;
const LFSR_FEEDBACK_MASK: u16 = 0b11;
const LFSR_SHORT_MODE_BIT: u16 = 6;
const LFSR_LONG_MODE_BIT: u16 = 14;

const NOISE_DIVISOR_BASE: u32 = 8;
const NOISE_DIVISORS: [u32; 8] = [8, 16, 32, 48, 64, 80, 96, 112];

pub(super) struct Channel4 {
    active: bool,

    nr41: u8,
    nr42: u8,
    nr43: u8,
    nr44: u8,

    length_counter: LengthCounter,
    length_enabled: bool,

    lfsr: u16,
    volume: u8,
    envelope_timer: u8,

    noise_timer: u32,
}

impl Channel4 {
    pub(super) fn new() -> Self {
        Self {
            active: false,

            nr41: 0,
            nr42: 0,
            nr43: 0,
            nr44: 0,

            length_counter: LengthCounter::default(),
            length_enabled: false,

            lfsr: 0x7FFF,
            volume: 0,
            envelope_timer: 0,

            noise_timer: 0,
        }
    }

    pub(super) fn read(&self, address: u16) -> u8 {
        match address {
            NR41_ADDRESS => NR41_READ_VALUE,
            NR42_ADDRESS => self.nr42 | NR42_READ_MASK,
            NR43_ADDRESS => self.nr43 | NR43_READ_MASK,
            NR44_ADDRESS => self.nr44 | NR44_READ_MASK,
            _ => 0xFF,
        }
    }

    pub(super) fn write(&mut self, address: u16, value: u8) {
        self.write_with_extra_length_clock(address, value, false);
    }

    pub(super) fn write_with_extra_length_clock(
        &mut self,
        address: u16,
        value: u8,
        extra_length_clock: bool,
    ) {
        match address {
            NR41_ADDRESS => {
                self.nr41 = value;
                self.length_counter
                    .load(u16::from(value & NR41_LENGTH_MASK));
            }
            NR42_ADDRESS => {
                self.nr42 = value;

                if !envelope::is_dac_enabled(value) {
                    self.active = false;
                }
            }
            NR43_ADDRESS => self.nr43 = value,
            NR44_ADDRESS => {
                let was_length_enabled = self.length_enabled;

                self.nr44 = value & !NR44_TRIGGER_MASK;
                self.length_enabled = value & NR44_LENGTH_ENABLE_MASK != 0;

                if !was_length_enabled
                    && self.length_enabled
                    && extra_length_clock
                    && self.length_counter.tick()
                {
                    self.active = false;
                }

                if value & NR44_TRIGGER_MASK != 0 {
                    self.trigger(extra_length_clock);
                }
            }
            _ => {}
        }
    }

    fn get_noise_period(&self) -> u32 {
        let clock_shift = u32::from((self.nr43 & NR43_CLOCK_SHIFT_MASK) >> 4);
        let divisor_code = u32::from(self.nr43 & NR43_DIVISOR_MASK);

        let divisor = if divisor_code == 0 {
            NOISE_DIVISOR_BASE
        } else {
            NOISE_DIVISORS[divisor_code as usize]
        };

        divisor << clock_shift
    }

    fn clock_lfsr(&mut self) {
        let feedback = (self.lfsr & LFSR_FEEDBACK_MASK).count_ones() as u16 & 1;

        self.lfsr >>= 1;
        self.lfsr |= feedback << LFSR_LONG_MODE_BIT;

        if self.nr43 & NR43_WIDTH_MODE_MASK != 0 {
            self.lfsr &= !(1 << LFSR_SHORT_MODE_BIT);
            self.lfsr |= feedback << LFSR_SHORT_MODE_BIT;
        }
    }

    pub(super) fn clock_envelope(&mut self) {
        envelope::clock(
            self.nr42,
            self.active,
            &mut self.volume,
            &mut self.envelope_timer,
        );
    }

    fn trigger(&mut self, extra_length_clock: bool) {
        self.active = envelope::is_dac_enabled(self.nr42);

        if self.length_counter.is_zero() {
            self.length_counter.reload();

            if self.length_enabled && extra_length_clock && self.length_counter.tick() {
                self.active = false;
            }
        }

        envelope::trigger(self.nr42, &mut self.volume, &mut self.envelope_timer);
        self.lfsr = LFSR_INITIAL_VALUE;
        self.noise_timer = self.get_noise_period();
    }

    pub(super) fn clock_length(&mut self) {
        if !self.length_enabled {
            return;
        }

        if self.length_counter.tick() {
            self.active = false;
        }
    }

    pub(super) fn is_active(&self) -> bool {
        self.active
    }

    pub(super) fn get_output(&self) -> u8 {
        if !self.active || !envelope::is_dac_enabled(self.nr42) || self.lfsr & 1 != 0 {
            return 0;
        }

        self.volume
    }

    pub(super) fn tick(&mut self) {
        if !self.active {
            return;
        }

        if self.noise_timer > 0 {
            self.noise_timer -= 1;
        }

        if self.noise_timer == 0 {
            self.noise_timer = self.get_noise_period();
            self.clock_lfsr();
        }
    }
}

impl Default for Channel4 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::apu::length_counter::LENGTH_COUNTER_MAX;

    use super::*;

    const MAX_VOLUME: u8 = 0x0F;

    #[test]
    fn length_register_reads_as_ones() {
        let channel = Channel4::new();

        assert_eq!(channel.read(NR41_ADDRESS), NR41_READ_VALUE);
    }

    #[test]
    fn trigger_bit_reads_as_zero() {
        let mut channel = Channel4::new();

        channel.write(NR44_ADDRESS, NR44_TRIGGER_MASK);

        assert_eq!(channel.read(NR44_ADDRESS), NR44_READ_MASK);
    }

    #[test]
    fn writing_length_register_loads_length_counter() {
        let mut channel = Channel4::new();
        let length = 1;

        channel.write(NR41_ADDRESS, length);

        assert_eq!(
            channel.length_counter.get_counter(),
            LENGTH_COUNTER_MAX - u16::from(length)
        );
    }

    #[test]
    fn trigger_activates_channel_when_dac_is_enabled() {
        let mut channel = Channel4::new();

        channel.write(NR42_ADDRESS, 0b1111_0000);
        channel.write(NR44_ADDRESS, NR44_TRIGGER_MASK);

        assert!(channel.is_active());
    }

    #[test]
    fn trigger_does_not_activate_channel_when_dac_is_disabled() {
        let mut channel = Channel4::new();

        channel.write(NR42_ADDRESS, 0);
        channel.write(NR44_ADDRESS, NR44_TRIGGER_MASK);

        assert!(!channel.is_active());
    }

    #[test]
    fn disabling_dac_deactivates_channel() {
        let mut channel = Channel4::new();

        channel.write(NR42_ADDRESS, 0b1111_0000);
        channel.write(NR44_ADDRESS, NR44_TRIGGER_MASK);

        assert!(channel.is_active());

        channel.write(NR42_ADDRESS, 0);

        assert!(!channel.is_active());
    }

    #[test]
    fn trigger_resets_linear_feedback_shift_register() {
        let mut channel = Channel4::new();

        channel.lfsr = 0;
        channel.write(NR44_ADDRESS, NR44_TRIGGER_MASK);

        assert_eq!(channel.lfsr, 0x7FFF);
    }

    #[test]
    fn lfsr_shifts_and_inserts_feedback_into_bit_14() {
        let mut channel = Channel4::new();
        channel.lfsr = 0b0000_0000_0000_0001;

        channel.clock_lfsr();

        assert_eq!(channel.lfsr, 1 << LFSR_LONG_MODE_BIT);
    }

    #[test]
    fn short_mode_also_updates_bit_6() {
        let mut channel = Channel4::new();
        channel.nr43 = NR43_WIDTH_MODE_MASK;
        channel.lfsr = 0b0000_0000_0000_0001;

        channel.clock_lfsr();

        assert_ne!(channel.lfsr & (1 << LFSR_SHORT_MODE_BIT), 0);
    }

    #[test]
    fn noise_timer_uses_divisor_and_clock_shift() {
        let mut channel = Channel4::new();
        channel.nr43 = 0;

        assert_eq!(channel.get_noise_period(), NOISE_DIVISOR_BASE);

        channel.nr43 = 0b0000_0011;

        assert_eq!(channel.get_noise_period(), NOISE_DIVISORS[3]);

        channel.nr43 = 0b0010_0011;

        assert_eq!(channel.get_noise_period(), NOISE_DIVISORS[3] << 2);
    }

    #[test]
    fn output_is_zero_when_channel_is_inactive() {
        let mut channel = Channel4::new();
        channel.volume = MAX_VOLUME;
        channel.lfsr = 0;

        assert_eq!(channel.get_output(), 0);
    }

    #[test]
    fn output_is_zero_when_dac_is_disabled() {
        let mut channel = Channel4::new();
        channel.nr42 = 0;
        channel.volume = MAX_VOLUME;
        channel.lfsr = 0;
        channel.active = true;

        assert_eq!(channel.get_output(), 0);
    }

    #[test]
    fn output_is_zero_when_lfsr_low_bit_is_set() {
        let mut channel = Channel4::new();

        channel.write(NR42_ADDRESS, 0b1111_0000);
        channel.write(NR44_ADDRESS, NR44_TRIGGER_MASK);
        channel.lfsr = 1;

        assert_eq!(channel.get_output(), 0);
    }

    #[test]
    fn output_returns_envelope_volume_when_lfsr_low_bit_is_clear() {
        let mut channel = Channel4::new();
        let initial_volume = MAX_VOLUME - 2;

        channel.write(NR42_ADDRESS, initial_volume << 4);
        channel.write(NR44_ADDRESS, NR44_TRIGGER_MASK);
        channel.lfsr = 0;

        assert_eq!(channel.get_output(), initial_volume);
    }

    #[test]
    fn lfsr_does_not_advance_before_noise_timer_expires() {
        let mut channel = Channel4::new();

        channel.write(NR42_ADDRESS, 0b1111_0000);
        channel.write(NR44_ADDRESS, NR44_TRIGGER_MASK);

        let initial_lfsr = channel.lfsr;
        let period = channel.get_noise_period();

        for _ in 0..period - 1 {
            channel.tick();
        }

        assert_eq!(channel.lfsr, initial_lfsr);
        assert_eq!(channel.noise_timer, 1);
    }

    #[test]
    fn lfsr_advances_when_noise_timer_expires() {
        let mut channel = Channel4::new();

        channel.write(NR42_ADDRESS, 0b1111_0000);
        channel.write(NR44_ADDRESS, NR44_TRIGGER_MASK);

        let initial_lfsr = channel.lfsr;
        let period = channel.get_noise_period();

        for _ in 0..period {
            channel.tick();
        }

        assert_ne!(channel.lfsr, initial_lfsr);
        assert_eq!(channel.noise_timer, period);
    }

    #[test]
    fn envelope_increases_volume_when_clocked() {
        let mut channel = Channel4::new();

        channel.write(NR42_ADDRESS, 0b0101_1001);
        channel.write(NR44_ADDRESS, NR44_TRIGGER_MASK);

        assert_eq!(channel.volume, 5);

        channel.clock_envelope();

        assert_eq!(channel.volume, 6);
    }

    #[test]
    fn envelope_decreases_volume_when_clocked() {
        let mut channel = Channel4::new();

        channel.write(NR42_ADDRESS, 0b0101_0001);
        channel.write(NR44_ADDRESS, NR44_TRIGGER_MASK);

        assert_eq!(channel.volume, 5);

        channel.clock_envelope();

        assert_eq!(channel.volume, 4);
    }

    #[test]
    fn channel_deactivates_when_length_counter_expires() {
        let mut channel = Channel4::new();

        channel.write(NR42_ADDRESS, 0b1111_0000);
        channel.write(NR41_ADDRESS, 0b0011_1111);
        channel.write(NR44_ADDRESS, NR44_TRIGGER_MASK | NR44_LENGTH_ENABLE_MASK);

        assert!(channel.is_active());

        for _ in 0..LENGTH_COUNTER_MAX {
            channel.clock_length();
        }

        assert!(!channel.is_active());
    }

    #[test]
    fn lfsr_shifts_and_updates_feedback_in_long_mode() {
        let mut channel = Channel4::new();

        channel.clock_lfsr();

        assert_eq!(channel.lfsr, LFSR_INITIAL_VALUE >> 1);
    }

    #[test]
    fn lfsr_updates_bit_6_in_short_mode() {
        let mut channel = Channel4::new();
        channel.nr43 = NR43_WIDTH_MODE_MASK;

        channel.clock_lfsr();

        let expected = (LFSR_INITIAL_VALUE >> 1) & !(1 << LFSR_SHORT_MODE_BIT);

        assert_eq!(channel.lfsr, expected);
    }

    #[test]
    fn noise_period_uses_divisor_code() {
        let mut channel = Channel4::new();
        channel.nr43 = 0b0000_0010;

        assert_eq!(channel.get_noise_period(), 32);
    }

    #[test]
    fn noise_period_applies_clock_shift() {
        let mut channel = Channel4::new();
        channel.nr43 = 0b0010_0001;

        assert_eq!(channel.get_noise_period(), 64);
    }

    #[test]
    fn output_is_zero_when_lfsr_bit_zero_is_set() {
        let mut channel = Channel4::new();

        channel.write(NR42_ADDRESS, 0b1111_0000);
        channel.write(NR44_ADDRESS, NR44_TRIGGER_MASK);

        assert_eq!(channel.lfsr & 1, 1);
        assert_eq!(channel.get_output(), 0);
    }

    #[test]
    fn output_returns_volume_when_lfsr_bit_zero_is_clear() {
        let mut channel = Channel4::new();

        channel.write(NR42_ADDRESS, 0b1111_0000);
        channel.write(NR44_ADDRESS, NR44_TRIGGER_MASK);
        channel.lfsr &= !1;

        assert_eq!(channel.get_output(), MAX_VOLUME);
    }

    #[test]
    fn tick_does_not_advance_lfsr_before_timer_expires() {
        let mut channel = Channel4::new();

        channel.write(NR42_ADDRESS, 0b1111_0000);
        channel.write(NR44_ADDRESS, NR44_TRIGGER_MASK);

        let initial_lfsr = channel.lfsr;
        let period = channel.get_noise_period();

        for _ in 0..period - 1 {
            channel.tick();
        }

        assert_eq!(channel.lfsr, initial_lfsr);
        assert_eq!(channel.noise_timer, 1);
    }

    #[test]
    fn tick_advances_lfsr_when_timer_expires() {
        let mut channel = Channel4::new();

        channel.write(NR42_ADDRESS, 0b1111_0000);
        channel.write(NR44_ADDRESS, NR44_TRIGGER_MASK);

        let initial_lfsr = channel.lfsr;
        let period = channel.get_noise_period();

        for _ in 0..period {
            channel.tick();
        }

        assert_ne!(channel.lfsr, initial_lfsr);
        assert_eq!(channel.noise_timer, period);
    }

    #[test]
    fn tick_does_not_advance_inactive_channel() {
        let mut channel = Channel4::new();

        let initial_lfsr = channel.lfsr;
        let initial_timer = channel.noise_timer;

        channel.tick();

        assert_eq!(channel.lfsr, initial_lfsr);
        assert_eq!(channel.noise_timer, initial_timer);
    }

    #[test]
    fn channel_becomes_inactive_when_length_expires() {
        let mut channel = Channel4::new();

        channel.write(NR41_ADDRESS, NR41_LENGTH_MASK);
        channel.write(NR42_ADDRESS, 0b1111_0000);
        channel.write(NR44_ADDRESS, NR44_TRIGGER_MASK | NR44_LENGTH_ENABLE_MASK);

        assert!(channel.is_active());

        channel.clock_length();

        assert!(!channel.is_active());
    }

    #[test]
    fn envelope_does_not_increase_above_max_volume() {
        let mut channel = Channel4::new();

        channel.write(NR42_ADDRESS, 0b1111_1001);
        channel.write(NR44_ADDRESS, NR44_TRIGGER_MASK);

        channel.clock_envelope();

        assert_eq!(channel.volume, MAX_VOLUME);
    }

    #[test]
    fn envelope_does_not_decrease_below_zero() {
        let mut channel = Channel4::new();

        channel.write(NR42_ADDRESS, 0b0000_0001);
        channel.write(NR44_ADDRESS, NR44_TRIGGER_MASK);

        channel.clock_envelope();

        assert_eq!(channel.volume, 0);
    }
}
