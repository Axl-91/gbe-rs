#![allow(dead_code)]

const NR10_ADDRESS: u16 = 0xFF10;
const NR11_ADDRESS: u16 = 0xFF11;
const NR12_ADDRESS: u16 = 0xFF12;
const NR13_ADDRESS: u16 = 0xFF13;
const NR14_ADDRESS: u16 = 0xFF14;

const NR10_READ_MASK: u8 = 0x80;
const NR11_READ_MASK: u8 = 0x3F;
const NR13_READ_VALUE: u8 = 0xFF;
const NR14_READ_MASK: u8 = 0xBF;

const NR14_TRIGGER_MASK: u8 = 0x80;
const NR12_DAC_MASK: u8 = 0xF8;

const FREQUENCY_MAX: u16 = 0x07FF;
const FREQUENCY_TIMER_MULTIPLIER: u16 = 4;
const DUTY_STEPS: u8 = 8;
const NR14_FREQUENCY_MASK: u8 = 0b0000_0111;

const NR11_DUTY_MASK: u8 = 0b1100_0000;
const NR11_DUTY_SHIFT: u32 = 6;
const DUTY_PATTERNS: [u8; 4] = [0b1000_0000, 0b1000_0001, 0b1110_0001, 0b0111_1110];

const NR12_ENVELOPE_DIRECTION_MASK: u8 = 0b0000_1000;
const NR12_ENVELOPE_PERIOD_MASK: u8 = 0b0000_0111;
const NR12_INITIAL_VOLUME_SHIFT: u32 = 4;
const ENVELOPE_MAX_VOLUME: u8 = 0x0F;

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
}

impl Channel1 {
    pub fn new() -> Self {
        Self {
            active: false,
            initial_output_suppressed: false,

            nr10: 0,
            nr11: 0,
            nr12: 0,
            nr13: 0,
            nr14: 0,

            frequency_timer: 0,
            duty_position: 0,

            envelope_volume: 0,
            envelope_timer: 0,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            NR10_ADDRESS => self.nr10 | NR10_READ_MASK,
            NR11_ADDRESS => self.nr11 | NR11_READ_MASK,
            NR12_ADDRESS => self.nr12,
            NR13_ADDRESS => NR13_READ_VALUE,
            NR14_ADDRESS => self.nr14 | NR14_READ_MASK,
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            NR10_ADDRESS => self.nr10 = value,
            NR11_ADDRESS => self.nr11 = value,
            NR12_ADDRESS => {
                self.nr12 = value;

                if !self.dac_enabled() {
                    self.active = false;
                }
            }
            NR13_ADDRESS => self.nr13 = value,
            NR14_ADDRESS => {
                self.nr14 = value & !NR14_TRIGGER_MASK;

                if value & NR14_TRIGGER_MASK != 0 {
                    self.trigger();
                }
            }
            _ => {}
        }
    }

    fn dac_enabled(&self) -> bool {
        self.nr12 & NR12_DAC_MASK != 0
    }

    fn envelope_period(&self) -> u8 {
        let period = self.nr12 & NR12_ENVELOPE_PERIOD_MASK;

        if period == 0 { 8 } else { period }
    }

    /// Triggers the channel and initializes its frequency timer and envelope.
    ///
    /// The channel becomes active only if its DAC is enabled.
    fn trigger(&mut self) {
        self.active = self.dac_enabled();
        self.frequency_timer = self.frequency_period();

        self.envelope_volume = self.nr12 >> NR12_INITIAL_VOLUME_SHIFT;
        self.envelope_timer = self.envelope_period();
        self.initial_output_suppressed = true;
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Advances the channel's frequency timer by one T-cycle.
    ///
    /// When the timer expires, reloads it and advances the duty-cycle position.
    pub fn tick(&mut self) {
        if !self.active {
            return;
        }

        self.frequency_timer -= 1;

        if self.frequency_timer == 0 {
            self.frequency_timer = self.frequency_period();
            self.duty_position = (self.duty_position + 1) % DUTY_STEPS;
            self.initial_output_suppressed = false;
        }
    }

    fn frequency(&self) -> u16 {
        let high = u16::from(self.nr14 & NR14_FREQUENCY_MASK);
        let low = u16::from(self.nr13);

        (high << u8::BITS) | low
    }

    /// Calculates the pulse timer period in T-cycles from the 11-bit frequency.
    fn frequency_period(&self) -> u16 {
        (FREQUENCY_MAX + 1 - self.frequency()) * FREQUENCY_TIMER_MULTIPLIER
    }

    /// Returns the high/low state of the selected duty pattern at the current phase.
    fn duty_output(&self) -> bool {
        let duty = usize::from((self.nr11 & NR11_DUTY_MASK) >> NR11_DUTY_SHIFT);
        let pattern = DUTY_PATTERNS[duty];
        let bit = (pattern >> self.duty_position) & 1;

        bit != 0
    }

    /// Returns the channel's current digital output level.
    ///
    /// The output is zero when the channel or DAC is disabled, or while the
    /// initial output is suppressed.
    pub fn output(&self) -> u8 {
        if !self.active || !self.dac_enabled() || self.initial_output_suppressed {
            return 0;
        }

        if self.duty_output() {
            self.envelope_volume
        } else {
            0
        }
    }

    /// Applies one envelope clock from the frame sequencer.
    ///
    /// Changes the current volume by one step in the configured direction,
    /// without allowing it to exceed the range 0–15.
    pub fn clock_envelope(&mut self) {
        if !self.active {
            return;
        }

        let period = self.nr12 & NR12_ENVELOPE_PERIOD_MASK;

        if period == 0 {
            return;
        }

        self.envelope_timer -= 1;

        if self.envelope_timer > 0 {
            return;
        }

        self.envelope_timer = period;

        let increasing = self.nr12 & NR12_ENVELOPE_DIRECTION_MASK != 0;

        if increasing && self.envelope_volume < ENVELOPE_MAX_VOLUME {
            self.envelope_volume += 1;
        } else if !increasing && self.envelope_volume > 0 {
            self.envelope_volume -= 1;
        }
    }
}

impl Default for Channel1 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        assert_eq!(channel.frequency(), expected);
    }

    #[test]
    fn inactive_channel_outputs_zero() {
        let channel = Channel1::new();

        assert_eq!(channel.output(), 0);
    }

    #[test]
    fn channel_outputs_zero_immediately_after_trigger() {
        let mut channel = Channel1::new();

        channel.write(NR12_ADDRESS, 0xF0);
        channel.write(NR11_ADDRESS, 0b1000_0000);
        channel.write(NR14_ADDRESS, NR14_TRIGGER_MASK | NR14_FREQUENCY_MASK);

        assert_eq!(channel.output(), 0);
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

        assert_eq!(channel.output(), expected);
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
    fn envelope_does_not_change_when_period_is_zero() {
        let mut channel = Channel1::new();

        channel.write(NR12_ADDRESS, 0b0101_1000);
        channel.write(NR14_ADDRESS, NR14_TRIGGER_MASK);

        for _ in 0..20 {
            channel.clock_envelope();
        }

        assert_eq!(channel.envelope_volume, 5);
    }
}
