use crate::apu::length_counter::LengthCounter;
use crate::apu::pulse;

const NR30_ADDRESS: u16 = 0xFF1A;
pub(super) const NR31_ADDRESS: u16 = 0xFF1B;
const NR32_ADDRESS: u16 = 0xFF1C;
const NR33_ADDRESS: u16 = 0xFF1D;
const NR34_ADDRESS: u16 = 0xFF1E;

const WAVE_RAM_START: u16 = 0xFF30;
const WAVE_RAM_END: u16 = 0xFF3F;

const NR30_DAC_MASK: u8 = 0b1000_0000;
const NR31_LENGTH_MASK: u8 = 0xFF;

const NR34_TRIGGER_MASK: u8 = 0b1000_0000;
const NR34_LENGTH_ENABLE_MASK: u8 = 0b0100_0000;

const NR30_READ_MASK: u8 = 0b0111_1111;
const NR31_READ_VALUE: u8 = 0xFF;
const NR32_READ_MASK: u8 = 0b1001_1111;
const NR33_READ_VALUE: u8 = 0xFF;
const NR34_READ_MASK: u8 = 0b1011_1111;

const WAVE_RAM_SIZE: usize = 16;
const WAVE_SAMPLE_COUNT: u8 = 32;
const WAVE_SAMPLE_MASK: u8 = 0x0F;

const NR32_OUTPUT_LEVEL_MASK: u8 = 0b0110_0000;
const NR32_OUTPUT_LEVEL_SHIFT: u32 = 5;

const NR32_OUTPUT_LEVEL_MUTE: u8 = 0;
const NR32_OUTPUT_LEVEL_FULL: u8 = 1;
const NR32_OUTPUT_LEVEL_HALF: u8 = 2;
const NR32_OUTPUT_LEVEL_QUARTER: u8 = 3;

const FREQUENCY_TIMER_BASE: u16 = 2048;
const FREQUENCY_TIMER_MULTIPLIER: u16 = 2;

const CHANNEL3_LENGTH_COUNTER_MAX: u16 = 256;

const WAVE_RAM_ACCESS_WINDOW_T_CYCLES: u8 = 2;
const WAVE_RAM_ACCESS_BLOCKED_T_CYCLES: u8 = WAVE_RAM_ACCESS_WINDOW_T_CYCLES + 1;
const WAVE_RAM_TRIGGER_DELAY_T_CYCLES: u16 = 6;

const WAVE_RAM_RETRIGGER_LOOKAHEAD_T_CYCLES: u16 = 2;

pub(super) struct Channel3 {
    active: bool,

    nr30: u8,
    nr31: u8,
    nr32: u8,
    nr33: u8,
    nr34: u8,

    wave_ram: [u8; WAVE_RAM_SIZE],

    frequency_timer: u16,
    wave_position: u8,

    length_counter: LengthCounter,
    length_enabled: bool,

    t_cycles_since_wave_ram_read: u8,
}

impl Channel3 {
    pub fn new() -> Self {
        Self {
            active: false,

            nr30: 0,
            nr31: 0,
            nr32: 0,
            nr33: 0,
            nr34: 0,

            wave_ram: [0; WAVE_RAM_SIZE],

            frequency_timer: 0,
            wave_position: 0,

            length_counter: LengthCounter::new(CHANNEL3_LENGTH_COUNTER_MAX),
            length_enabled: false,

            t_cycles_since_wave_ram_read: WAVE_RAM_ACCESS_BLOCKED_T_CYCLES,
        }
    }

    pub(super) fn reset(&mut self) {
        let length = self.length_counter.get_counter();
        let wave_ram = self.wave_ram;

        *self = Self::new();
        self.nr30 = 0x00;
        self.nr31 = 0x00;
        self.nr32 = 0x00;
        self.nr33 = 0x00;
        self.nr34 = 0x00;

        self.length_counter.set_counter(length);
        self.wave_ram = wave_ram;
    }

    fn can_access_wave_ram(&self) -> bool {
        self.t_cycles_since_wave_ram_read < WAVE_RAM_ACCESS_WINDOW_T_CYCLES
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            NR30_ADDRESS => self.nr30 | NR30_READ_MASK,
            NR31_ADDRESS => NR31_READ_VALUE,
            NR32_ADDRESS => self.nr32 | NR32_READ_MASK,
            NR33_ADDRESS => NR33_READ_VALUE,
            NR34_ADDRESS => self.nr34 | NR34_READ_MASK,
            WAVE_RAM_START..=WAVE_RAM_END => {
                if !self.active {
                    let index = usize::from(address - WAVE_RAM_START);
                    self.wave_ram[index]
                } else if self.can_access_wave_ram() {
                    let index = usize::from(self.wave_position / 2);
                    self.wave_ram[index]
                } else {
                    0xFF
                }
            }
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
            NR30_ADDRESS => {
                self.nr30 = value;

                if !self.is_dac_enabled() {
                    self.active = false;
                }
            }
            NR31_ADDRESS => {
                self.nr31 = value;
                self.length_counter
                    .load(u16::from(value & NR31_LENGTH_MASK));
            }
            NR32_ADDRESS => self.nr32 = value,
            NR33_ADDRESS => self.nr33 = value,
            NR34_ADDRESS => {
                let was_length_enabled = self.length_enabled;

                self.nr34 = value & !NR34_TRIGGER_MASK;
                self.length_enabled = value & NR34_LENGTH_ENABLE_MASK != 0;

                if !was_length_enabled
                    && self.length_enabled
                    && extra_length_clock
                    && self.length_counter.tick()
                {
                    self.active = false;
                }

                if value & NR34_TRIGGER_MASK != 0 {
                    self.trigger(extra_length_clock);
                }
            }
            WAVE_RAM_START..=WAVE_RAM_END => {
                if !self.active {
                    let index = usize::from(address - WAVE_RAM_START);
                    self.wave_ram[index] = value;
                } else if self.can_access_wave_ram() {
                    let index = usize::from(self.wave_position / 2);
                    self.wave_ram[index] = value;
                }
            }
            _ => {}
        }
    }

    fn is_dac_enabled(&self) -> bool {
        self.nr30 & NR30_DAC_MASK != 0
    }

    fn get_frequency(&self) -> u16 {
        pulse::get_frequency(self.nr33, self.nr34)
    }

    fn get_frequency_period(&self) -> u16 {
        (FREQUENCY_TIMER_BASE - self.get_frequency()) * FREQUENCY_TIMER_MULTIPLIER
    }

    fn get_wave_sample(&self) -> u8 {
        let byte_index = usize::from(self.wave_position / 2);
        let sample_byte = self.wave_ram[byte_index];

        let sample = if self.wave_position.is_multiple_of(2) {
            sample_byte >> 4
        } else {
            sample_byte & WAVE_SAMPLE_MASK
        };

        sample & WAVE_SAMPLE_MASK
    }

    pub fn get_output(&self) -> u8 {
        if !self.active {
            return 0;
        }

        let output_level = (self.nr32 & NR32_OUTPUT_LEVEL_MASK) >> NR32_OUTPUT_LEVEL_SHIFT;

        match output_level {
            NR32_OUTPUT_LEVEL_MUTE => 0,
            NR32_OUTPUT_LEVEL_FULL => self.get_wave_sample(),
            NR32_OUTPUT_LEVEL_HALF => self.get_wave_sample() >> 1,
            NR32_OUTPUT_LEVEL_QUARTER => self.get_wave_sample() >> 2,
            _ => unreachable!("NR32 output level is a two-bit value"),
        }
    }

    pub fn clock_length(&mut self) {
        if !self.length_enabled {
            return;
        }

        if self.length_counter.tick() {
            self.active = false;
        }
    }

    fn corrupt_wave_ram_on_retrigger(&mut self) {
        // The retrigger lands right before the next read, so the byte being
        // fetched belongs to the next sample (wave_position has not advanced yet).
        let next_position = (self.wave_position + 1) % WAVE_SAMPLE_COUNT;
        let current_byte = usize::from(next_position / 2);

        if current_byte < 4 {
            self.wave_ram[0] = self.wave_ram[current_byte];
            return;
        }

        let aligned_byte = current_byte & !0b11;

        let corrupted_bytes = [
            self.wave_ram[aligned_byte],
            self.wave_ram[aligned_byte + 1],
            self.wave_ram[aligned_byte + 2],
            self.wave_ram[aligned_byte + 3],
        ];

        self.wave_ram[..corrupted_bytes.len()].copy_from_slice(&corrupted_bytes);
    }

    fn trigger(&mut self, extra_length_clock: bool) {
        let was_active = self.active;

        if was_active
            && self.is_dac_enabled()
            && self.frequency_timer <= WAVE_RAM_RETRIGGER_LOOKAHEAD_T_CYCLES
        {
            self.corrupt_wave_ram_on_retrigger();
        }

        self.active = self.is_dac_enabled();

        if self.length_counter.is_zero() {
            self.length_counter.reload();

            if self.length_enabled && extra_length_clock && self.length_counter.tick() {
                self.active = false;
            }
        }

        self.frequency_timer = self.get_frequency_period() + WAVE_RAM_TRIGGER_DELAY_T_CYCLES;
        self.wave_position = 0;
        self.t_cycles_since_wave_ram_read = WAVE_RAM_ACCESS_BLOCKED_T_CYCLES;
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Advances the channel by one T-cycle.
    pub fn tick(&mut self) {
        if !self.active {
            return;
        }

        self.frequency_timer -= 1;

        if self.frequency_timer == 0 {
            self.frequency_timer = self.get_frequency_period();
            self.wave_position = (self.wave_position + 1) % WAVE_SAMPLE_COUNT;
            self.t_cycles_since_wave_ram_read = 0;
        } else {
            self.t_cycles_since_wave_ram_read = self.t_cycles_since_wave_ram_read.saturating_add(1);
        }
    }

    pub(super) fn write_length_counter(&mut self, value: u8) {
        self.length_counter.load(u16::from(value));
    }

    #[cfg(test)]
    pub(super) fn get_length_counter_for_test(&self) -> u16 {
        self.length_counter.get_counter()
    }
}

impl Default for Channel3 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::apu::Apu;

    use super::*;

    const NR34_FREQUENCY_MASK: u8 = 0b0000_0111;

    #[test]
    fn registers_can_be_written_and_read() {
        let mut channel = Channel3::new();

        const DAC_ENABLED: u8 = NR30_DAC_MASK;
        const OUTPUT_LEVEL: u8 = 0b0110_0000;
        const FREQUENCY_LOW: u8 = 0x34;
        const FREQUENCY_HIGH: u8 = 0x05;

        channel.write(NR30_ADDRESS, DAC_ENABLED);
        channel.write(NR31_ADDRESS, 0x20);
        channel.write(NR32_ADDRESS, OUTPUT_LEVEL);
        channel.write(NR33_ADDRESS, FREQUENCY_LOW);
        channel.write(NR34_ADDRESS, FREQUENCY_HIGH);

        assert_eq!(channel.read(NR30_ADDRESS) & NR30_DAC_MASK, DAC_ENABLED);
        assert_eq!(
            channel.read(NR32_ADDRESS) & NR32_OUTPUT_LEVEL_MASK,
            OUTPUT_LEVEL
        );
        assert_eq!(channel.read(NR33_ADDRESS), NR33_READ_VALUE);
        assert_eq!(
            channel.read(NR34_ADDRESS),
            (FREQUENCY_HIGH & !NR34_TRIGGER_MASK) | NR34_READ_MASK
        );
    }

    #[test]
    fn dac_disabled_deactivates_channel() {
        let mut channel = Channel3::new();

        channel.write(NR30_ADDRESS, NR30_DAC_MASK);
        channel.write(NR34_ADDRESS, NR34_TRIGGER_MASK);

        assert!(channel.is_active());

        channel.write(NR30_ADDRESS, 0);

        assert!(!channel.is_active());
    }

    #[test]
    fn trigger_activates_channel_when_dac_is_enabled() {
        let mut channel = Channel3::new();

        channel.write(NR30_ADDRESS, NR30_DAC_MASK);
        channel.write(NR34_ADDRESS, NR34_TRIGGER_MASK);

        assert!(channel.is_active());
    }

    #[test]
    fn trigger_does_not_activate_channel_when_dac_is_disabled() {
        let mut channel = Channel3::new();

        channel.write(NR34_ADDRESS, NR34_TRIGGER_MASK);

        assert!(!channel.is_active());
    }

    #[test]
    fn wave_ram_can_be_written_and_read() {
        let mut channel = Channel3::new();

        const FIRST_WAVE_BYTE: u8 = 0x12;
        const LAST_WAVE_BYTE: u8 = 0xAB;

        channel.write(WAVE_RAM_START, FIRST_WAVE_BYTE);
        channel.write(WAVE_RAM_END, LAST_WAVE_BYTE);

        assert_eq!(channel.read(WAVE_RAM_START), FIRST_WAVE_BYTE);
        assert_eq!(channel.read(WAVE_RAM_END), LAST_WAVE_BYTE);
    }

    #[test]
    fn trigger_resets_wave_position() {
        let mut channel = Channel3::new();

        channel.wave_position = 17;

        channel.write(NR30_ADDRESS, NR30_DAC_MASK);
        channel.write(NR34_ADDRESS, NR34_TRIGGER_MASK);

        assert_eq!(channel.wave_position, 0);
    }

    #[test]
    fn output_level_reduces_sample_amplitude() {
        let mut channel = Channel3::new();

        channel.write(NR30_ADDRESS, NR30_DAC_MASK);
        channel.write(WAVE_RAM_START, 0xC0);
        channel.write(NR34_ADDRESS, NR34_TRIGGER_MASK);

        channel.write(
            NR32_ADDRESS,
            NR32_OUTPUT_LEVEL_FULL << NR32_OUTPUT_LEVEL_SHIFT,
        );
        assert_eq!(channel.get_output(), 0x0C);

        channel.write(
            NR32_ADDRESS,
            NR32_OUTPUT_LEVEL_HALF << NR32_OUTPUT_LEVEL_SHIFT,
        );
        assert_eq!(channel.get_output(), 0x06);

        channel.write(
            NR32_ADDRESS,
            NR32_OUTPUT_LEVEL_QUARTER << NR32_OUTPUT_LEVEL_SHIFT,
        );
        assert_eq!(channel.get_output(), 0x03);

        channel.write(
            NR32_ADDRESS,
            NR32_OUTPUT_LEVEL_MUTE << NR32_OUTPUT_LEVEL_SHIFT,
        );
        assert_eq!(channel.get_output(), 0);
    }

    #[test]
    fn tick_does_not_advance_when_channel_is_inactive() {
        let mut channel = Channel3::new();
        let initial_timer = channel.frequency_timer;

        channel.tick();

        assert_eq!(channel.wave_position, 0);
        assert_eq!(channel.frequency_timer, initial_timer);
    }

    #[test]
    fn frequency_timer_period_is_maximum_at_zero_frequency() {
        let mut channel = Channel3::new();

        channel.write(NR30_ADDRESS, NR30_DAC_MASK);
        channel.write(NR33_ADDRESS, 0);
        channel.write(NR34_ADDRESS, NR34_TRIGGER_MASK);

        assert_eq!(channel.get_frequency_period(), 4096);
    }

    #[test]
    fn frequency_timer_period_is_minimum_at_maximum_frequency() {
        let mut channel = Channel3::new();

        channel.write(NR30_ADDRESS, NR30_DAC_MASK);
        channel.write(NR33_ADDRESS, u8::MAX);
        channel.write(NR34_ADDRESS, NR34_TRIGGER_MASK | NR34_FREQUENCY_MASK);

        assert_eq!(channel.get_frequency_period(), 2);
    }

    #[test]
    fn channel3_length_counter_can_be_written_when_apu_is_disabled() {
        const NR52_ADDRESS: u16 = 0xFF26;
        const NR31_ADDRESS: u16 = 0xFF1B;

        const APU_DISABLE: u8 = 0x00;
        const LENGTH_VALUE: u8 = 100;
        const EXPECTED_LENGTH_COUNTER: u16 = CHANNEL3_LENGTH_COUNTER_MAX - LENGTH_VALUE as u16;

        let mut apu = Apu::new();

        apu.write(NR52_ADDRESS, APU_DISABLE);
        apu.write(NR31_ADDRESS, LENGTH_VALUE);

        assert_eq!(
            apu.channel3.get_length_counter_for_test(),
            EXPECTED_LENGTH_COUNTER
        );
        assert_eq!(apu.read(NR52_ADDRESS), 0x70);
    }

    #[test]
    fn reset_preserves_length_counter() {
        const LENGTH_VALUE: u8 = 100;
        const EXPECTED_LENGTH: u16 = CHANNEL3_LENGTH_COUNTER_MAX - LENGTH_VALUE as u16;

        let mut channel = Channel3::new();
        channel.write_length_counter(LENGTH_VALUE);

        channel.reset();

        assert_eq!(channel.get_length_counter_for_test(), EXPECTED_LENGTH);
    }

    #[test]
    fn reset_preserves_wave_ram() {
        const WAVE_RAM_ADDRESS: u16 = 0xFF30;
        const WAVE_RAM_VALUE: u8 = 0xA5;

        let mut channel = Channel3::new();

        channel.write(WAVE_RAM_ADDRESS, WAVE_RAM_VALUE);
        channel.reset();

        assert_eq!(channel.read(WAVE_RAM_ADDRESS), WAVE_RAM_VALUE);
    }
}
