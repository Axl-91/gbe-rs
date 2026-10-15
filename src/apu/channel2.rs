#![allow(dead_code)]

use crate::apu::{
    length_counter::{LENGTH_COUNTER_MAX, LengthCounter},
    pulse,
};

const NR21_ADDRESS: u16 = 0xFF16;
const NR22_ADDRESS: u16 = 0xFF17;
const NR23_ADDRESS: u16 = 0xFF18;
const NR24_ADDRESS: u16 = 0xFF19;

const NR21_READ_MASK: u8 = 0x3F;
const NR23_READ_VALUE: u8 = 0xFF;
const NR24_READ_MASK: u8 = 0xBF;

const NR24_TRIGGER_MASK: u8 = 0x80;

const NR21_LENGTH_MASK: u8 = 0x3F;
const NR24_LENGTH_ENABLE_MASK: u8 = 0x40;

pub(super) struct Channel2 {
    active: bool,
    initial_output_suppressed: bool,

    nr21: u8,
    nr22: u8,
    nr23: u8,
    nr24: u8,

    frequency_timer: u16,
    duty_position: u8,

    envelope_volume: u8,
    envelope_timer: u8,

    length_counter: LengthCounter,
    length_enabled: bool,
}

impl Channel2 {
    pub fn new() -> Self {
        Self {
            active: false,
            initial_output_suppressed: false,

            nr21: 0,
            nr22: 0,
            nr23: 0,
            nr24: 0,

            frequency_timer: 0,
            duty_position: 0,

            envelope_volume: 0,
            envelope_timer: 0,

            length_counter: LengthCounter::new(LENGTH_COUNTER_MAX),
            length_enabled: false,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            NR21_ADDRESS => self.nr21 | NR21_READ_MASK,
            NR22_ADDRESS => self.nr22,
            NR23_ADDRESS => NR23_READ_VALUE,
            NR24_ADDRESS => self.nr24 | NR24_READ_MASK,
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        self.write_with_extra_length_clock(address, value, false);
    }

    pub(super) fn write_with_extra_length_clock(
        &mut self,
        address: u16,
        value: u8,
        extra_length_clock: bool,
    ) {
        match address {
            NR21_ADDRESS => {
                self.nr21 = value;
                self.length_counter
                    .load(u16::from(value & NR21_LENGTH_MASK));
            }
            NR22_ADDRESS => {
                self.nr22 = value;

                if !self.is_dac_enabled() {
                    self.active = false;
                }
            }
            NR23_ADDRESS => self.nr23 = value,
            NR24_ADDRESS => {
                let was_length_enabled = self.length_enabled;

                self.nr24 = value & !NR24_TRIGGER_MASK;
                self.length_enabled = value & NR24_LENGTH_ENABLE_MASK != 0;

                if !was_length_enabled
                    && self.length_enabled
                    && extra_length_clock
                    && self.length_counter.tick()
                {
                    self.active = false;
                }

                if value & NR24_TRIGGER_MASK != 0 {
                    self.trigger(extra_length_clock);
                }
            }
            _ => {}
        }
    }

    fn is_dac_enabled(&self) -> bool {
        super::envelope::is_dac_enabled(self.nr22)
    }

    fn get_frequency(&self) -> u16 {
        pulse::get_frequency(self.nr23, self.nr24)
    }

    fn get_frequency_period(&self) -> u16 {
        pulse::get_frequency_period(self.get_frequency())
    }

    fn trigger(&mut self, extra_length_clock: bool) {
        self.active = self.is_dac_enabled();

        if self.length_counter.is_zero() {
            self.length_counter.reload();

            if self.length_enabled && extra_length_clock {
                self.length_counter.tick();
            }
        }

        self.frequency_timer = self.get_frequency_period();

        super::envelope::trigger(
            self.nr22,
            &mut self.envelope_volume,
            &mut self.envelope_timer,
        );

        self.initial_output_suppressed = true;
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn clock_envelope(&mut self) {
        super::envelope::clock(
            self.nr22,
            self.active,
            &mut self.envelope_volume,
            &mut self.envelope_timer,
        );
    }

    pub fn clock_length(&mut self) {
        if !self.length_enabled {
            return;
        }

        if self.length_counter.tick() {
            self.active = false;
        }
    }

    pub fn tick(&mut self) {
        let frequency_period = self.get_frequency_period();

        pulse::tick(
            self.active,
            &mut self.frequency_timer,
            frequency_period,
            &mut self.duty_position,
            &mut self.initial_output_suppressed,
        );
    }

    fn get_duty_output(&self) -> bool {
        pulse::get_duty_output(self.nr21, self.duty_position)
    }

    pub fn get_output(&self) -> u8 {
        if !self.active || !self.is_dac_enabled() || self.initial_output_suppressed {
            return 0;
        }

        if self.get_duty_output() {
            self.envelope_volume
        } else {
            0
        }
    }
}

impl Default for Channel2 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NR22_DAC_MASK: u8 = 0xF8;
    const FREQUENCY_MAX: u16 = 0x07FF;
    const FREQUENCY_TIMER_MULTIPLIER: u16 = 4;
    const NR24_FREQUENCY_MASK: u8 = 0b0000_0111;
    const NR22_INITIAL_VOLUME_SHIFT: u32 = 4;

    #[test]
    fn registers_return_expected_values_after_writes() {
        let mut channel = Channel2::new();

        let nr21_value = 0b1000_1010;
        let nr22_value = 0b1100_0101;
        let nr23_value = 0b0110_1010;
        let nr24_value = 0b0100_0011;

        channel.write(NR21_ADDRESS, nr21_value);
        channel.write(NR22_ADDRESS, nr22_value);
        channel.write(NR23_ADDRESS, nr23_value);
        channel.write(NR24_ADDRESS, nr24_value);

        assert_eq!(channel.read(NR21_ADDRESS), nr21_value | NR21_READ_MASK);
        assert_eq!(channel.read(NR22_ADDRESS), nr22_value);
        assert_eq!(channel.read(NR23_ADDRESS), NR23_READ_VALUE);
        assert_eq!(channel.read(NR24_ADDRESS), nr24_value | NR24_READ_MASK);
    }

    #[test]
    fn channel_starts_inactive() {
        let channel = Channel2::new();

        assert!(!channel.is_active());
    }

    #[test]
    fn trigger_activates_channel_when_dac_is_enabled() {
        let mut channel = Channel2::new();

        channel.write(NR22_ADDRESS, NR22_DAC_MASK);
        channel.write(NR24_ADDRESS, NR24_TRIGGER_MASK);

        assert!(channel.is_active());
    }

    #[test]
    fn trigger_does_not_activate_channel_when_dac_is_disabled() {
        let mut channel = Channel2::new();

        channel.write(NR22_ADDRESS, 0);
        channel.write(NR24_ADDRESS, NR24_TRIGGER_MASK);

        assert!(!channel.is_active());
    }

    #[test]
    fn disabling_dac_deactivates_channel() {
        let mut channel = Channel2::new();

        channel.write(NR22_ADDRESS, NR22_DAC_MASK);
        channel.write(NR24_ADDRESS, NR24_TRIGGER_MASK);
        channel.write(NR22_ADDRESS, 0);

        assert!(!channel.is_active());
    }

    #[test]
    fn frequency_uses_low_byte_and_three_high_bits() {
        let mut channel = Channel2::new();

        channel.write(NR23_ADDRESS, 0xA5);
        channel.write(NR24_ADDRESS, 0x05);

        let expected = (u16::from(0x05_u8) << u8::BITS) | u16::from(0xA5_u8);

        assert_eq!(channel.get_frequency(), expected);
    }

    #[test]
    fn frequency_period_is_calculated_from_frequency() {
        let mut channel = Channel2::new();

        let frequency = 1024;

        channel.write(NR23_ADDRESS, frequency as u8);
        channel.write(NR24_ADDRESS, (frequency >> u8::BITS) as u8);

        let expected = (FREQUENCY_MAX + 1 - frequency) * FREQUENCY_TIMER_MULTIPLIER;

        assert_eq!(channel.get_frequency_period(), expected);
    }

    #[test]
    fn inactive_channel_outputs_zero() {
        let channel = Channel2::new();

        assert_eq!(channel.get_output(), 0);
    }

    #[test]
    fn channel_outputs_zero_immediately_after_trigger() {
        let mut channel = Channel2::new();

        let nr22 = 0xF0;

        channel.write(NR22_ADDRESS, nr22);
        channel.write(NR21_ADDRESS, 0b1000_0000);
        channel.write(NR24_ADDRESS, NR24_TRIGGER_MASK);

        assert_eq!(channel.get_output(), 0);
    }

    #[test]
    fn active_channel_outputs_volume_on_high_duty_step() {
        let mut channel = Channel2::new();

        let nr22 = 0xF0;
        let frequency = 2047;

        channel.write(NR22_ADDRESS, nr22);
        channel.write(NR21_ADDRESS, 0b1000_0000);
        channel.write(NR23_ADDRESS, frequency as u8);
        channel.write(
            NR24_ADDRESS,
            ((frequency >> u8::BITS) as u8 & NR24_FREQUENCY_MASK) | NR24_TRIGGER_MASK,
        );

        for _ in 0..(FREQUENCY_TIMER_MULTIPLIER * 5) {
            channel.tick();
        }

        let expected = nr22 >> 4;

        assert_eq!(channel.get_output(), expected);
    }

    #[test]
    fn trigger_initializes_envelope_volume() {
        let mut channel = Channel2::new();
        let initial_volume = 5;

        let nr22 = initial_volume << NR22_INITIAL_VOLUME_SHIFT;

        channel.write(NR22_ADDRESS, nr22);
        channel.write(NR24_ADDRESS, NR24_TRIGGER_MASK);

        assert_eq!(channel.envelope_volume, initial_volume);
    }

    #[test]
    fn trigger_initializes_envelope_timer() {
        let mut channel = Channel2::new();
        let envelope_period = 3;

        let nr22 = (5 << NR22_INITIAL_VOLUME_SHIFT) | NR22_DAC_MASK | envelope_period;

        channel.write(NR22_ADDRESS, nr22);
        channel.write(NR24_ADDRESS, NR24_TRIGGER_MASK);

        assert_eq!(channel.envelope_timer, envelope_period);
    }

    #[test]
    fn trigger_initializes_envelope_timer_to_eight_when_period_is_zero() {
        let mut channel = Channel2::new();

        let nr22 = (5 << NR22_INITIAL_VOLUME_SHIFT) | NR22_DAC_MASK;

        channel.write(NR22_ADDRESS, nr22);
        channel.write(NR24_ADDRESS, NR24_TRIGGER_MASK);

        assert_eq!(channel.envelope_timer, 8);
    }
}
