use crate::apu::{envelope, length_counter::LengthCounter, pulse};

const NR10_ADDRESS: u16 = 0xFF10;
pub(super) const NR11_ADDRESS: u16 = 0xFF11;
const NR12_ADDRESS: u16 = 0xFF12;
const NR13_ADDRESS: u16 = 0xFF13;
const NR14_ADDRESS: u16 = 0xFF14;

const NR10_READ_MASK: u8 = 0x80;
const NR11_READ_MASK: u8 = 0x3F;
const NR13_READ_VALUE: u8 = 0xFF;
const NR14_READ_MASK: u8 = 0xBF;

const NR14_TRIGGER_MASK: u8 = 0x80;

const FREQUENCY_MAX: u16 = 0x07FF;
const NR14_FREQUENCY_MASK: u8 = 0b0000_0111;

const NR10_SWEEP_PERIOD_MASK: u8 = 0b0111_0000;
const NR10_SWEEP_PERIOD_SHIFT: u32 = 4;
const NR10_SWEEP_DIRECTION_MASK: u8 = 0b0000_1000;
const NR10_SWEEP_SHIFT_MASK: u8 = 0b0000_0111;

const NR11_LENGTH_MASK: u8 = 0x3F;
const NR14_LENGTH_ENABLE_MASK: u8 = 0x40;

const SWEEP_PERIOD_ZERO: u8 = 8;

pub(super) struct Channel1 {
    active: bool,
    initial_output_suppressed: bool,

    nr10: u8,
    nr11: u8,
    nr12: u8,
    nr13: u8,
    nr14: u8,

    frequency_timer: u16,
    duty_position: u8,

    envelope_volume: u8,
    envelope_timer: u8,

    sweep_timer: u8,
    sweep_shadow_frequency: u16,
    sweep_enabled: bool,

    length_counter: LengthCounter,
    length_enabled: bool,
}

impl Channel1 {
    pub(super) fn new() -> Self {
        Self {
            active: false,
            initial_output_suppressed: false,

            nr10: 0x00,
            nr11: 0b1000_0000,
            nr12: 0xF3,
            nr13: 0x00,
            nr14: 0x00,

            frequency_timer: 0,
            duty_position: 0,

            envelope_volume: 0,
            envelope_timer: 0,

            sweep_timer: 0,
            sweep_shadow_frequency: 0,
            sweep_enabled: false,

            length_counter: LengthCounter::default(),
            length_enabled: false,
        }
    }

    pub(super) fn reset(&mut self) {
        let length = self.length_counter.get_counter();

        *self = Self::new();
        self.nr10 = 0x00;
        self.nr11 = 0x00;
        self.nr12 = 0x00;
        self.nr13 = 0x00;
        self.nr14 = 0x00;

        self.length_counter.set_counter(length);
    }

    pub(super) fn read(&self, address: u16) -> u8 {
        match address {
            NR10_ADDRESS => self.nr10 | NR10_READ_MASK,
            NR11_ADDRESS => self.nr11 | NR11_READ_MASK,
            NR12_ADDRESS => self.nr12,
            NR13_ADDRESS => NR13_READ_VALUE,
            NR14_ADDRESS => self.nr14 | NR14_READ_MASK,
            _ => 0xFF,
        }
    }

    #[cfg(test)]
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
            NR10_ADDRESS => self.nr10 = value,
            NR11_ADDRESS => {
                self.nr11 = value;
                self.length_counter
                    .load(u16::from(value & NR11_LENGTH_MASK));
            }
            NR12_ADDRESS => {
                self.nr12 = value;

                if !self.is_dac_enabled() {
                    self.active = false;
                }
            }
            NR13_ADDRESS => self.nr13 = value,
            NR14_ADDRESS => {
                let was_length_enabled = self.length_enabled;

                self.nr14 = value & !NR14_TRIGGER_MASK;
                self.length_enabled = value & NR14_LENGTH_ENABLE_MASK != 0;

                if !was_length_enabled
                    && self.length_enabled
                    && extra_length_clock
                    && self.length_counter.tick()
                {
                    self.active = false;
                }

                if value & NR14_TRIGGER_MASK != 0 {
                    self.trigger(extra_length_clock);
                }
            }
            _ => {}
        }
    }

    fn get_sweep_period(&self) -> u8 {
        (self.nr10 & NR10_SWEEP_PERIOD_MASK) >> NR10_SWEEP_PERIOD_SHIFT
    }

    fn get_sweep_direction_decrease(&self) -> bool {
        self.nr10 & NR10_SWEEP_DIRECTION_MASK != 0
    }

    fn get_sweep_shift(&self) -> u8 {
        self.nr10 & NR10_SWEEP_SHIFT_MASK
    }

    /// Writes the 11-bit frequency to NR13 and NR14.
    fn set_frequency(&mut self, frequency: u16) {
        self.nr13 = frequency as u8;

        self.nr14 = (self.nr14 & !NR14_FREQUENCY_MASK)
            | ((frequency >> u8::BITS) as u8 & NR14_FREQUENCY_MASK);
    }

    fn is_dac_enabled(&self) -> bool {
        envelope::is_dac_enabled(self.nr12)
    }

    pub(super) fn is_active(&self) -> bool {
        self.active
    }

    fn get_frequency(&self) -> u16 {
        pulse::get_frequency(self.nr13, self.nr14)
    }

    #[cfg(test)]
    pub(super) fn get_frequency_for_test(&self) -> u16 {
        self.get_frequency()
    }

    /// Calculates the pulse timer period in T-cycles from the 11-bit frequency.
    fn get_frequency_period(&self) -> u16 {
        pulse::get_frequency_period(self.get_frequency())
    }

    /// Calculates the next frequency according to the sweep settings.
    ///
    /// Returns `None` when the shift is zero or the result exceeds the
    /// maximum frequency.
    fn calculate_sweep_frequency(&self) -> Option<u16> {
        let shift = self.get_sweep_shift();

        if shift == 0 {
            return None;
        }

        let frequency = self.sweep_shadow_frequency;
        let delta = frequency >> shift;

        let new_frequency = if self.get_sweep_direction_decrease() {
            frequency - delta
        } else {
            frequency + delta
        };

        if new_frequency > FREQUENCY_MAX {
            return None;
        }

        Some(new_frequency)
    }

    /// Advances the sweep and applies a new frequency when its timer expires.
    pub(super) fn clock_sweep(&mut self) {
        if !self.tick_sweep_timer() {
            return;
        }

        if self.get_sweep_shift() == 0 {
            return;
        }

        let Some(frequency) = self.calculate_sweep_frequency() else {
            self.active = false;
            return;
        };

        self.sweep_shadow_frequency = frequency;
        self.set_frequency(frequency);

        if self.calculate_sweep_frequency().is_none() {
            self.active = false;
        }
    }

    pub(super) fn clock_length(&mut self) {
        if !self.length_enabled {
            return;
        }

        if self.length_counter.tick() {
            self.active = false;
        }
    }

    /// Triggers the channel and initializes its frequency timer and envelope.
    ///
    /// The channel becomes active only if its DAC is enabled.
    fn trigger(&mut self, extra_length_clock: bool) {
        self.active = self.is_dac_enabled();

        if self.length_counter.is_zero() {
            self.length_counter.reload();

            if self.length_enabled && extra_length_clock {
                self.length_counter.tick();
            }
        }
        self.frequency_timer = self.get_frequency_period();

        self.sweep_shadow_frequency = self.get_frequency();

        let sweep_period = self.get_sweep_period();
        self.sweep_timer = if sweep_period == 0 {
            SWEEP_PERIOD_ZERO
        } else {
            sweep_period
        };

        self.sweep_enabled = sweep_period != 0 || self.get_sweep_shift() != 0;

        if self.get_sweep_shift() != 0 && self.calculate_sweep_frequency().is_none() {
            self.active = false;
        }

        super::envelope::trigger(
            self.nr12,
            &mut self.envelope_volume,
            &mut self.envelope_timer,
        );
        self.initial_output_suppressed = true;
    }

    /// Returns the high/low state of the selected duty pattern at the current phase.
    fn get_duty_output(&self) -> bool {
        pulse::get_duty_output(self.nr11, self.duty_position)
    }

    /// Returns the channel's current digital output level.
    ///
    /// The output is zero when the channel or DAC is disabled, or while the
    /// initial output is suppressed.
    pub(super) fn get_output(&self) -> u8 {
        if !self.active || !self.is_dac_enabled() || self.initial_output_suppressed {
            return 0;
        }

        if self.get_duty_output() {
            self.envelope_volume
        } else {
            0
        }
    }

    /// Applies one envelope clock from the frame sequencer.
    ///
    /// Changes the current volume by one step in the configured direction,
    /// without allowing it to exceed the range 0–15.
    pub(super) fn clock_envelope(&mut self) {
        super::envelope::clock(
            self.nr12,
            self.active,
            &mut self.envelope_volume,
            &mut self.envelope_timer,
        );
    }

    pub(super) fn write_length_counter(&mut self, value: u8) {
        self.length_counter
            .load(u16::from(value & NR11_LENGTH_MASK));
    }

    #[cfg(test)]
    pub(super) fn get_length_counter_for_test(&self) -> u16 {
        self.length_counter.get_counter()
    }

    /// Advances the sweep timer by one frame sequencer clock.
    ///
    /// Returns `true` when the timer expires and must perform a sweep calculation.
    fn tick_sweep_timer(&mut self) -> bool {
        if !self.sweep_enabled {
            return false;
        }

        self.sweep_timer -= 1;

        if self.sweep_timer > 0 {
            return false;
        }

        let period = self.get_sweep_period();

        self.sweep_timer = if period == 0 {
            SWEEP_PERIOD_ZERO
        } else {
            period
        };

        true
    }

    /// Advances the channel's frequency timer by one T-cycle.
    ///
    /// When the timer expires, reloads it and advances the duty-cycle position.
    pub(super) fn tick(&mut self) {
        let frequency_period = self.get_frequency_period();

        pulse::tick(
            self.active,
            &mut self.frequency_timer,
            frequency_period,
            &mut self.duty_position,
            &mut self.initial_output_suppressed,
        );
    }
}

impl Default for Channel1 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::apu::{Apu, length_counter::LENGTH_COUNTER_MAX};

    use super::*;

    const NR12_DAC_MASK: u8 = 0xF8;
    const FREQUENCY_TIMER_MULTIPLIER: u16 = 4;
    const ENVELOPE_MAX_VOLUME: u8 = 0x0F;

    #[test]
    fn registers_return_expected_values_after_writes() {
        let mut channel = Channel1::new();

        let nr10_value = 0b0101_0011;
        let nr11_value = 0b1000_1010;
        let nr12_value = 0b1100_0101;
        let nr13_value = 0b0110_1010;
        let nr14_value = 0b0100_0011;

        channel.write(NR10_ADDRESS, nr10_value);
        channel.write(NR11_ADDRESS, nr11_value);
        channel.write(NR12_ADDRESS, nr12_value);
        channel.write(NR13_ADDRESS, nr13_value);
        channel.write(NR14_ADDRESS, nr14_value);

        assert_eq!(channel.read(NR10_ADDRESS), nr10_value | NR10_READ_MASK);
        assert_eq!(channel.read(NR11_ADDRESS), nr11_value | NR11_READ_MASK);
        assert_eq!(channel.read(NR12_ADDRESS), nr12_value);
        assert_eq!(channel.read(NR13_ADDRESS), NR13_READ_VALUE);
        assert_eq!(channel.read(NR14_ADDRESS), nr14_value | NR14_READ_MASK);
    }

    #[test]
    fn channel_starts_inactive() {
        let channel = Channel1::new();

        assert!(!channel.is_active());
    }

    #[test]
    fn trigger_activates_channel_when_dac_is_enabled() {
        let mut channel = Channel1::new();

        channel.write(NR12_ADDRESS, NR12_DAC_MASK);
        channel.write(NR14_ADDRESS, NR14_TRIGGER_MASK);

        assert!(channel.is_active());
    }

    #[test]
    fn trigger_does_not_activate_channel_when_dac_is_disabled() {
        let mut channel = Channel1::new();

        channel.write(NR12_ADDRESS, 0);
        channel.write(NR14_ADDRESS, NR14_TRIGGER_MASK);

        assert!(!channel.is_active());
    }

    #[test]
    fn disabling_dac_deactivates_channel() {
        let mut channel = Channel1::new();

        channel.write(NR12_ADDRESS, NR12_DAC_MASK);
        channel.write(NR14_ADDRESS, NR14_TRIGGER_MASK);
        channel.write(NR12_ADDRESS, 0);

        assert!(!channel.is_active());
    }

    #[test]
    fn frequency_uses_low_byte_and_three_high_bits() {
        let mut channel = Channel1::new();

        channel.write(NR13_ADDRESS, 0xA5);
        channel.write(NR14_ADDRESS, 0x05);

        let expected = (u16::from(0x05_u8) << u8::BITS) | u16::from(0xA5_u8);

        assert_eq!(channel.get_frequency(), expected);
    }

    #[test]
    fn inactive_channel_outputs_zero() {
        let channel = Channel1::new();

        assert_eq!(channel.get_output(), 0);
    }

    #[test]
    fn channel_outputs_zero_immediately_after_trigger() {
        let mut channel = Channel1::new();

        channel.write(NR12_ADDRESS, 0xF0);
        channel.write(NR11_ADDRESS, 0b1000_0000);
        channel.write(NR14_ADDRESS, NR14_TRIGGER_MASK | NR14_FREQUENCY_MASK);

        assert_eq!(channel.get_output(), 0);
    }

    #[test]
    fn active_channel_outputs_volume_on_high_duty_step() {
        let mut channel = Channel1::new();
        let nr12 = 0xF0;

        channel.write(NR12_ADDRESS, nr12);
        channel.write(NR11_ADDRESS, 0b1000_0000);
        channel.write(NR13_ADDRESS, u8::MAX);
        channel.write(NR14_ADDRESS, NR14_TRIGGER_MASK | NR14_FREQUENCY_MASK);

        for _ in 0..(FREQUENCY_TIMER_MULTIPLIER * 5) {
            channel.tick();
        }

        let expected = nr12 >> 4;

        assert_eq!(channel.get_output(), expected);
    }

    #[test]
    fn envelope_increases_volume_when_configured() {
        let mut channel = Channel1::new();

        channel.write(NR12_ADDRESS, 0b0101_1001);
        channel.write(NR14_ADDRESS, NR14_TRIGGER_MASK);

        for _ in 0..2 {
            channel.clock_envelope();
        }

        assert_eq!(channel.envelope_volume, 7);
    }

    #[test]
    fn envelope_decreases_volume_when_configured() {
        let mut channel = Channel1::new();

        channel.write(NR12_ADDRESS, 0b1000_0010);
        channel.write(NR14_ADDRESS, NR14_TRIGGER_MASK);

        channel.clock_envelope();
        channel.clock_envelope();

        assert_eq!(channel.envelope_volume, 7);
    }

    #[test]
    fn envelope_does_not_exceed_maximum_volume() {
        let mut channel = Channel1::new();

        channel.write(NR12_ADDRESS, 0b1111_1001);
        channel.write(NR14_ADDRESS, NR14_TRIGGER_MASK);

        for _ in 0..20 {
            channel.clock_envelope();
        }

        assert_eq!(channel.envelope_volume, ENVELOPE_MAX_VOLUME);
    }

    #[test]
    fn envelope_does_not_go_below_zero() {
        let mut channel = Channel1::new();

        channel.write(NR12_ADDRESS, 0b0000_0010);
        channel.write(NR14_ADDRESS, NR14_TRIGGER_MASK);

        for _ in 0..20 {
            channel.clock_envelope();
        }

        assert_eq!(channel.envelope_volume, 0);
    }

    #[test]
    fn envelope_increases_volume_after_eight_clocks_when_period_is_zero() {
        let mut channel = Channel1::new();

        channel.write(NR12_ADDRESS, 0b0101_1000);
        channel.write(NR14_ADDRESS, NR14_TRIGGER_MASK);

        for _ in 0..7 {
            channel.clock_envelope();
        }

        assert_eq!(channel.envelope_volume, 5);

        channel.clock_envelope();

        assert_eq!(channel.envelope_volume, 6);
        assert_eq!(channel.envelope_timer, 8);
    }

    #[test]
    fn sweep_period_uses_bits_six_to_four() {
        let mut channel = Channel1::new();

        let nr10 = 0b0101_0011;
        channel.write(NR10_ADDRESS, nr10);

        assert_eq!(channel.get_sweep_period(), 5);
    }

    #[test]
    fn sweep_direction_is_decrease_when_configured() {
        let mut channel = Channel1::new();

        channel.write(NR10_ADDRESS, 0b0000_1000);

        assert!(channel.get_sweep_direction_decrease());
    }

    #[test]
    fn sweep_direction_is_increase_when_configured() {
        let mut channel = Channel1::new();

        channel.write(NR10_ADDRESS, 0);

        assert!(!channel.get_sweep_direction_decrease());
    }

    #[test]
    fn sweep_shift_uses_low_three_bits() {
        let mut channel = Channel1::new();

        let nr10 = 0b0000_0101;
        channel.write(NR10_ADDRESS, nr10);

        assert_eq!(channel.get_sweep_shift(), 5);
    }

    #[test]
    fn sweep_calculates_increased_frequency() {
        let mut channel = Channel1::new();
        let frequency = 1024_u16;

        let nr14_value = ((frequency >> u8::BITS) as u8) | NR14_TRIGGER_MASK;

        channel.write(NR13_ADDRESS, frequency as u8);
        channel.write(NR10_ADDRESS, 0b0001_0001);
        channel.write(NR12_ADDRESS, NR12_DAC_MASK);
        channel.write(NR14_ADDRESS, nr14_value);

        assert_eq!(
            channel.calculate_sweep_frequency(),
            Some(frequency + (frequency >> 1)),
        );
    }

    #[test]
    fn sweep_calculates_decreased_frequency() {
        let mut channel = Channel1::new();
        let frequency = 1024_u16;

        let nr10_value = 0b0000_1010;
        let nr12_value = NR12_DAC_MASK;
        let nr13_value = frequency as u8;
        let nr14_value = ((frequency >> u8::BITS) as u8) | NR14_TRIGGER_MASK;

        channel.write(NR10_ADDRESS, nr10_value);
        channel.write(NR12_ADDRESS, nr12_value);
        channel.write(NR13_ADDRESS, nr13_value);
        channel.write(NR14_ADDRESS, nr14_value);

        assert_eq!(
            channel.calculate_sweep_frequency(),
            Some(frequency - (frequency >> 2)),
        );
    }

    #[test]
    fn sweep_returns_none_when_frequency_overflows() {
        let mut channel = Channel1::new();
        let frequency = FREQUENCY_MAX;

        let nr10_value = 0b0000_0001;
        let nr12_value = NR12_DAC_MASK;
        let nr13_value = frequency as u8;
        let nr14_value = ((frequency >> u8::BITS) as u8) | NR14_TRIGGER_MASK;

        channel.write(NR10_ADDRESS, nr10_value);
        channel.write(NR12_ADDRESS, nr12_value);
        channel.write(NR13_ADDRESS, nr13_value);
        channel.write(NR14_ADDRESS, nr14_value);

        assert_eq!(channel.calculate_sweep_frequency(), None);
    }

    #[test]
    fn sweep_returns_none_when_shift_is_zero() {
        let mut channel = Channel1::new();

        channel.write(NR10_ADDRESS, 0b0001_0000);

        assert_eq!(channel.calculate_sweep_frequency(), None);
    }

    #[test]
    fn trigger_initializes_sweep_shadow_frequency() {
        let mut channel = Channel1::new();
        let frequency = 1024_u16;
        let nr14_value = ((frequency >> u8::BITS) as u8) | NR14_TRIGGER_MASK;

        channel.write(NR13_ADDRESS, frequency as u8);
        channel.write(NR10_ADDRESS, 0b0001_0001);
        channel.write(NR12_ADDRESS, NR12_DAC_MASK);
        channel.write(NR14_ADDRESS, nr14_value);

        assert_eq!(channel.sweep_shadow_frequency, frequency);
    }

    #[test]
    fn trigger_initializes_sweep_timer_from_period() {
        let mut channel = Channel1::new();

        channel.write(NR10_ADDRESS, 0b0010_0001);
        channel.write(NR12_ADDRESS, NR12_DAC_MASK);
        channel.write(NR14_ADDRESS, NR14_TRIGGER_MASK);

        assert_eq!(channel.sweep_timer, 2);
        assert!(channel.sweep_enabled);
    }

    #[test]
    fn trigger_uses_eight_when_sweep_period_is_zero() {
        let mut channel = Channel1::new();

        channel.write(NR10_ADDRESS, 0b0000_0001);
        channel.write(NR12_ADDRESS, NR12_DAC_MASK);
        channel.write(NR14_ADDRESS, NR14_TRIGGER_MASK);

        assert_eq!(channel.sweep_timer, 8);
        assert!(channel.sweep_enabled);
    }

    #[test]
    fn sweep_is_disabled_when_period_and_shift_are_zero() {
        let mut channel = Channel1::new();

        channel.write(NR10_ADDRESS, 0);
        channel.write(NR12_ADDRESS, NR12_DAC_MASK);
        channel.write(NR14_ADDRESS, NR14_TRIGGER_MASK);

        assert!(!channel.sweep_enabled);
    }

    #[test]
    fn sweep_timer_does_not_expire_before_configured_period() {
        let mut channel = Channel1::new();

        channel.write(NR10_ADDRESS, 0b0010_0001);
        channel.write(NR12_ADDRESS, NR12_DAC_MASK);
        channel.write(NR14_ADDRESS, NR14_TRIGGER_MASK);

        assert!(!channel.tick_sweep_timer());
        assert_eq!(channel.sweep_timer, 1);
    }

    #[test]
    fn sweep_timer_expires_after_configured_period() {
        let mut channel = Channel1::new();

        channel.write(NR10_ADDRESS, 0b0010_0001);
        channel.write(NR12_ADDRESS, NR12_DAC_MASK);
        channel.write(NR14_ADDRESS, NR14_TRIGGER_MASK);

        assert!(!channel.tick_sweep_timer());
        assert!(channel.tick_sweep_timer());
        assert_eq!(channel.sweep_timer, 2);
    }

    #[test]
    fn sweep_timer_uses_eight_when_period_is_zero() {
        let mut channel = Channel1::new();

        channel.write(NR10_ADDRESS, 0b0000_0001);
        channel.write(NR12_ADDRESS, NR12_DAC_MASK);
        channel.write(NR14_ADDRESS, NR14_TRIGGER_MASK);

        for _ in 0..7 {
            assert!(!channel.tick_sweep_timer());
        }

        assert!(channel.tick_sweep_timer());
        assert_eq!(channel.sweep_timer, SWEEP_PERIOD_ZERO);
    }

    #[test]
    fn disabled_sweep_timer_does_not_advance() {
        let mut channel = Channel1::new();

        assert!(!channel.tick_sweep_timer());
        assert_eq!(channel.sweep_timer, 0);
    }

    #[test]
    fn sweep_updates_frequency_when_timer_expires() {
        let mut channel = Channel1::new();
        let frequency = 1024_u16;

        let nr10_value = 0b0001_0001;
        let nr12_value = NR12_DAC_MASK;
        let nr13_value = frequency as u8;
        let nr14_value = ((frequency >> u8::BITS) as u8) | NR14_TRIGGER_MASK;

        channel.write(NR10_ADDRESS, nr10_value);
        channel.write(NR12_ADDRESS, nr12_value);
        channel.write(NR13_ADDRESS, nr13_value);
        channel.write(NR14_ADDRESS, nr14_value);

        assert_eq!(channel.get_frequency(), frequency);

        channel.clock_sweep();

        assert_eq!(channel.get_frequency(), frequency + (frequency >> 1));
        assert_eq!(channel.sweep_shadow_frequency, frequency + (frequency >> 1),);
    }

    #[test]
    fn sweep_deactivates_channel_when_frequency_overflows() {
        let mut channel = Channel1::new();
        let frequency = FREQUENCY_MAX - (FREQUENCY_MAX >> 1);

        let nr10_value = 0b0001_0001;
        let nr12_value = NR12_DAC_MASK;
        let nr13_value = frequency as u8;
        let nr14_value = ((frequency >> u8::BITS) as u8) | NR14_TRIGGER_MASK;

        channel.write(NR10_ADDRESS, nr10_value);
        channel.write(NR12_ADDRESS, nr12_value);
        channel.write(NR13_ADDRESS, nr13_value);
        channel.write(NR14_ADDRESS, nr14_value);

        assert!(channel.is_active());

        channel.clock_sweep();

        assert!(!channel.is_active());
    }

    #[test]
    fn sweep_does_not_update_frequency_when_shift_is_zero() {
        let mut channel = Channel1::new();
        let frequency = 1024_u16;

        let nr10_value = 0b0001_0000;
        let nr12_value = NR12_DAC_MASK;
        let nr13_value = frequency as u8;
        let nr14_value = ((frequency >> u8::BITS) as u8) | NR14_TRIGGER_MASK;

        channel.write(NR10_ADDRESS, nr10_value);
        channel.write(NR12_ADDRESS, nr12_value);
        channel.write(NR13_ADDRESS, nr13_value);
        channel.write(NR14_ADDRESS, nr14_value);

        channel.clock_sweep();

        assert_eq!(channel.get_frequency(), frequency);
        assert_eq!(channel.sweep_shadow_frequency, frequency);
        assert!(channel.is_active());
    }

    #[test]
    fn sweep_deactivates_channel_when_trigger_frequency_overflows() {
        let mut channel = Channel1::new();
        let frequency = FREQUENCY_MAX;

        channel.write(NR10_ADDRESS, 0b0001_0001);
        channel.write(NR12_ADDRESS, NR12_DAC_MASK);
        channel.write(NR13_ADDRESS, frequency as u8);
        channel.write(
            NR14_ADDRESS,
            ((frequency >> u8::BITS) as u8 & NR14_FREQUENCY_MASK) | NR14_TRIGGER_MASK,
        );

        assert!(!channel.is_active());
    }

    #[test]
    fn channel1_length_counter_can_be_written_when_apu_is_disabled() {
        const NR52_ADDRESS: u16 = 0xFF26;
        const NR11_ADDRESS: u16 = 0xFF11;

        const APU_DISABLE: u8 = 0x00;
        const LENGTH_VALUE: u8 = 10;
        const EXPECTED_LENGTH_COUNTER: u16 = LENGTH_COUNTER_MAX - LENGTH_VALUE as u16;

        let mut apu = Apu::new();

        apu.write(NR52_ADDRESS, APU_DISABLE);
        apu.write(NR11_ADDRESS, LENGTH_VALUE);

        assert_eq!(
            apu.channel1.get_length_counter_for_test(),
            EXPECTED_LENGTH_COUNTER
        );

        assert_eq!(apu.read(NR52_ADDRESS), 0x70);
    }

    #[test]
    fn reset_preserves_length_counter() {
        const LENGTH_VALUE: u8 = 10;
        const EXPECTED_LENGTH: u16 = LENGTH_COUNTER_MAX - LENGTH_VALUE as u16;

        let mut channel = Channel1::new();
        channel.write_length_counter(LENGTH_VALUE);

        channel.reset();

        assert_eq!(channel.get_length_counter_for_test(), EXPECTED_LENGTH);
    }
}
