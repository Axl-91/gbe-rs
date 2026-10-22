//! Game Boy Audio Processing Unit (APU) implementation.
//!
//! Generates audio by managing the sound channels and their timing.
//! Coordinates channel updates through the frame sequencer and provides
//! the interface for audio register access.

mod channel1;
mod channel2;
mod channel3;
mod channel4;
mod envelope;
mod frame_sequencer;
mod length_counter;
mod pulse;

use channel1::Channel1;
use frame_sequencer::FrameSequencer;

use crate::apu::{
    channel1::NR11_ADDRESS,
    channel2::{Channel2, NR21_ADDRESS},
    channel3::{Channel3, NR31_ADDRESS},
    channel4::{Channel4, NR41_ADDRESS},
};

const NR52_ADDRESS: u16 = 0xFF26;
const NR50_ADDRESS: u16 = 0xFF24;
const NR51_ADDRESS: u16 = 0xFF25;

const NR52_UNUSED_BITS: u8 = 0b0111_0000;
const APU_ENABLE_MASK: u8 = 0b1000_0000;

const CHANNEL1_REGISTERS_START: u16 = 0xFF10;
const CHANNEL1_REGISTERS_END: u16 = 0xFF14;

const CHANNEL2_REGISTERS_START: u16 = 0xFF16;
const CHANNEL2_REGISTERS_END: u16 = 0xFF19;

const CHANNEL3_REGISTERS_START: u16 = 0xFF1A;
const CHANNEL3_REGISTERS_END: u16 = 0xFF1E;

const CHANNEL4_REGISTERS_START: u16 = 0xFF20;
const CHANNEL4_REGISTERS_END: u16 = 0xFF23;

const WAVE_RAM_START: u16 = 0xFF30;
const WAVE_RAM_END: u16 = 0xFF3F;

const SWEEP_STEP_1: u8 = 2;
const SWEEP_STEP_2: u8 = 6;
const ENVELOPE_STEP: u8 = 7;

const NR51_RIGHT_SHIFT: u8 = 0;
const NR51_LEFT_SHIFT: u8 = 4;

const NR50_VOLUME_MASK: u8 = 0b0000_0111;
const NR50_LEFT_VOLUME_SHIFT: u32 = 4;

const APU_CLOCK_HZ: u32 = 4_194_304;
const AUDIO_SAMPLE_RATE: u32 = 44_100;

/// Game Boy Audio Processing Unit.
pub struct Apu {
    enabled: bool,

    nr50: u8,
    nr51: u8,

    frame_sequencer: FrameSequencer,
    channel1: Channel1,
    channel2: Channel2,
    channel3: Channel3,
    channel4: Channel4,

    sample_cycles: u32,
    sample_buffer: Vec<(f32, f32)>,
}

impl Apu {
    pub fn new() -> Self {
        Self {
            enabled: true,

            nr50: 0x77,
            nr51: 0xF3,

            frame_sequencer: FrameSequencer::new(),
            channel1: Channel1::new(),
            channel2: Channel2::new(),
            channel3: Channel3::new(),
            channel4: Channel4::new(),

            sample_cycles: 0,
            sample_buffer: Vec::new(),
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            CHANNEL1_REGISTERS_START..=CHANNEL1_REGISTERS_END => self.channel1.read(address),
            CHANNEL2_REGISTERS_START..=CHANNEL2_REGISTERS_END => self.channel2.read(address),
            CHANNEL3_REGISTERS_START..=CHANNEL3_REGISTERS_END => self.channel3.read(address),
            CHANNEL4_REGISTERS_START..=CHANNEL4_REGISTERS_END => self.channel4.read(address),
            NR50_ADDRESS if self.enabled => self.nr50,
            NR51_ADDRESS if self.enabled => self.nr51,
            NR50_ADDRESS | NR51_ADDRESS => 0,
            NR52_ADDRESS => {
                let enabled = if self.enabled { APU_ENABLE_MASK } else { 0 };

                enabled | NR52_UNUSED_BITS | self.get_channel_status()
            }
            WAVE_RAM_START..=WAVE_RAM_END => self.channel3.read(address),
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            CHANNEL1_REGISTERS_START..=CHANNEL1_REGISTERS_END if self.enabled => {
                let extra_length_clock = self.frame_sequencer.next_step_skips_length_clock();

                self.channel1
                    .write_with_extra_length_clock(address, value, extra_length_clock);
            }
            CHANNEL1_REGISTERS_START..=CHANNEL1_REGISTERS_END
                if !self.enabled && address == NR11_ADDRESS =>
            {
                self.channel1.write_length_counter(value);
            }
            CHANNEL2_REGISTERS_START..=CHANNEL2_REGISTERS_END if self.enabled => {
                let extra_length_clock = self.frame_sequencer.next_step_skips_length_clock();
                self.channel2
                    .write_with_extra_length_clock(address, value, extra_length_clock);
            }
            CHANNEL2_REGISTERS_START..=CHANNEL2_REGISTERS_END
                if !self.enabled && address == NR21_ADDRESS =>
            {
                self.channel2.write_length_counter(value);
            }
            CHANNEL3_REGISTERS_START..=CHANNEL3_REGISTERS_END if self.enabled => {
                let extra_length_clock = self.frame_sequencer.next_step_skips_length_clock();

                self.channel3
                    .write_with_extra_length_clock(address, value, extra_length_clock);
            }
            CHANNEL3_REGISTERS_START..=CHANNEL3_REGISTERS_END
                if !self.enabled && address == NR31_ADDRESS =>
            {
                self.channel3.write_length_counter(value);
            }
            CHANNEL4_REGISTERS_START..=CHANNEL4_REGISTERS_END if self.enabled => {
                let extra_length_clock = self.frame_sequencer.next_step_skips_length_clock();

                self.channel4
                    .write_with_extra_length_clock(address, value, extra_length_clock);
            }
            CHANNEL4_REGISTERS_START..=CHANNEL4_REGISTERS_END
                if !self.enabled && address == NR41_ADDRESS =>
            {
                self.channel4.write_length_counter(value);
            }
            WAVE_RAM_START..=WAVE_RAM_END if self.enabled => {
                self.channel3.write(address, value);
            }
            NR50_ADDRESS if self.enabled => self.nr50 = value,
            NR51_ADDRESS if self.enabled => self.nr51 = value,
            NR52_ADDRESS => {
                let should_enable = value & APU_ENABLE_MASK != 0;

                if self.enabled && !should_enable {
                    self.channel1.reset();
                    self.channel2.reset();
                    self.channel3.reset();
                    self.channel4.reset();

                    self.nr50 = 0x00;
                    self.nr51 = 0x00;
                }

                if !self.enabled && should_enable {
                    self.frame_sequencer.power_on();
                }

                self.enabled = should_enable;
            }
            _ => {}
        }
    }

    fn get_channel_status(&self) -> u8 {
        u8::from(self.channel1.is_active())
            | (u8::from(self.channel2.is_active()) << 1)
            | (u8::from(self.channel3.is_active()) << 2)
            | (u8::from(self.channel4.is_active()) << 3)
    }

    /// Advances the APU by one T-cycle.
    pub fn tick(&mut self) {
        // Always runs: its phase follows the system divider, not the APU power.
        let frame_step_elapsed = self.frame_sequencer.tick();

        if self.enabled {
            self.channel1.tick();
            self.channel2.tick();
            self.channel3.tick();
            self.channel4.tick();

            if frame_step_elapsed {
                let step = self.frame_sequencer.get_step();

                if step.is_multiple_of(2) {
                    self.channel1.clock_length();
                    self.channel2.clock_length();
                    self.channel3.clock_length();
                    self.channel4.clock_length();
                }

                if step == SWEEP_STEP_1 || step == SWEEP_STEP_2 {
                    self.channel1.clock_sweep();
                }

                if step == ENVELOPE_STEP {
                    self.channel1.clock_envelope();
                    self.channel2.clock_envelope();
                    self.channel4.clock_envelope();
                }
            }
        }

        self.sample_cycles += AUDIO_SAMPLE_RATE;

        if self.sample_cycles >= APU_CLOCK_HZ {
            self.sample_cycles -= APU_CLOCK_HZ;

            let (left, right) = if self.enabled {
                self.get_mixed_output()
            } else {
                (0, 0)
            };

            self.sample_buffer
                .push((left as f32 / 960.0, right as f32 / 960.0));
        }
    }

    /// Returns the generated audio samples and clears the internal buffer.
    pub fn take_samples(&mut self) -> Vec<(f32, f32)> {
        std::mem::take(&mut self.sample_buffer)
    }

    fn get_channel_outputs(&self) -> [u8; 4] {
        [
            self.channel1.get_output(),
            self.channel2.get_output(),
            self.channel3.get_output(),
            self.channel4.get_output(),
        ]
    }

    fn get_mixed_output(&self) -> (u16, u16) {
        let outputs = self.get_channel_outputs();
        let mut left = 0_u16;
        let mut right = 0_u16;

        for (channel, output) in outputs.into_iter().enumerate() {
            let mask = 1 << channel;

            if self.nr51 & (mask << NR51_LEFT_SHIFT) != 0 {
                left += u16::from(output);
            }

            if self.nr51 & (mask << NR51_RIGHT_SHIFT) != 0 {
                right += u16::from(output);
            }
        }

        let left_volume = u16::from((self.nr50 >> NR50_LEFT_VOLUME_SHIFT) & NR50_VOLUME_MASK);
        let right_volume = u16::from(self.nr50 & NR50_VOLUME_MASK);

        (left * (left_volume + 1), right * (right_volume + 1))
    }
}

impl Default for Apu {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_channel1(apu: &mut Apu) {
        const NR11_ADDRESS: u16 = 0xFF11;
        const NR12_ADDRESS: u16 = 0xFF12;
        const NR13_ADDRESS: u16 = 0xFF13;
        const NR14_ADDRESS: u16 = 0xFF14;

        const DUTY_50_PERCENT: u8 = 0b1100_0000;
        const MAX_VOLUME: u8 = 0xF0;
        const FREQUENCY_LOW: u8 = 0xFF;
        const TRIGGER: u8 = 0x80;
        const FREQUENCY_HIGH: u8 = 0x07;

        apu.write(NR11_ADDRESS, DUTY_50_PERCENT);
        apu.write(NR12_ADDRESS, MAX_VOLUME);
        apu.write(NR13_ADDRESS, FREQUENCY_LOW);
        apu.write(NR14_ADDRESS, TRIGGER | FREQUENCY_HIGH);

        for _ in 0..4 {
            apu.tick();
        }
    }

    fn setup_channel2(apu: &mut Apu) {
        const NR21_ADDRESS: u16 = 0xFF16;
        const NR22_ADDRESS: u16 = 0xFF17;
        const NR23_ADDRESS: u16 = 0xFF18;
        const NR24_ADDRESS: u16 = 0xFF19;

        const DUTY_50_PERCENT: u8 = 0b1100_0000;
        const MAX_VOLUME: u8 = 0xF0;
        const FREQUENCY_LOW: u8 = 0xFF;
        const FREQUENCY_HIGH: u8 = 0x07;
        const TRIGGER: u8 = 0x80;

        apu.write(NR21_ADDRESS, DUTY_50_PERCENT);
        apu.write(NR22_ADDRESS, MAX_VOLUME);
        apu.write(NR23_ADDRESS, FREQUENCY_LOW);
        apu.write(NR24_ADDRESS, TRIGGER | FREQUENCY_HIGH);

        for _ in 0..4 {
            apu.tick();
        }
    }

    fn setup_channel3(apu: &mut Apu) {
        const NR30_ADDRESS: u16 = 0xFF1A;
        const NR32_ADDRESS: u16 = 0xFF1C;
        const NR34_ADDRESS: u16 = 0xFF1E;
        const WAVE_RAM_START: u16 = 0xFF30;

        const DAC_ENABLE: u8 = 0x80;
        const OUTPUT_LEVEL_FULL: u8 = 0b0110_0000;
        const TRIGGER: u8 = 0x80;
        const MAX_SAMPLE: u8 = 0xFF;

        apu.write(NR30_ADDRESS, DAC_ENABLE);
        apu.write(NR32_ADDRESS, OUTPUT_LEVEL_FULL);
        apu.write(WAVE_RAM_START, MAX_SAMPLE);
        apu.write(NR34_ADDRESS, TRIGGER);
    }

    fn setup_channel4(apu: &mut Apu) {
        const NR42_ADDRESS: u16 = 0xFF21;
        const NR44_ADDRESS: u16 = 0xFF23;
        const MAX_VOLUME: u8 = 0xF0;
        const TRIGGER: u8 = 0x80;

        apu.write(NR42_ADDRESS, MAX_VOLUME);
        apu.write(NR44_ADDRESS, TRIGGER);

        // Force a known LFSR output state for this routing test.
        apu.channel4.lfsr &= !1;
    }

    #[test]
    fn nr52_returns_disabled_channels_when_apu_is_enabled() {
        let apu = Apu::new();

        assert_eq!(apu.read(NR52_ADDRESS), APU_ENABLE_MASK | NR52_UNUSED_BITS);
    }

    #[test]
    fn nr52_returns_disabled_channels_when_apu_is_disabled() {
        let mut apu = Apu::new();

        apu.write(NR52_ADDRESS, 0);

        assert_eq!(apu.read(NR52_ADDRESS), NR52_UNUSED_BITS);
    }

    #[test]
    fn nr52_can_enable_apu_after_being_disabled() {
        let mut apu = Apu::new();

        apu.write(NR52_ADDRESS, 0);
        apu.write(NR52_ADDRESS, APU_ENABLE_MASK);

        assert_eq!(apu.read(NR52_ADDRESS), APU_ENABLE_MASK | NR52_UNUSED_BITS);
    }

    #[test]
    fn writing_unimplemented_apu_register_does_not_change_nr52() {
        let mut apu = Apu::new();

        apu.write(NR52_ADDRESS - 1, 0);

        assert_eq!(apu.read(NR52_ADDRESS), APU_ENABLE_MASK | NR52_UNUSED_BITS);
    }

    #[test]
    fn frame_sequencer_clocks_channel1_sweep_on_step_2() {
        const FRAME_SEQUENCER_PERIOD: u16 = 8192;

        const NR10_ADDRESS: u16 = 0xFF10;
        const NR12_ADDRESS: u16 = 0xFF12;
        const NR13_ADDRESS: u16 = 0xFF13;
        const NR14_ADDRESS: u16 = 0xFF14;

        const NR12_DAC_MASK: u8 = 0xF8;
        const NR14_TRIGGER_MASK: u8 = 0x80;
        const NR14_FREQUENCY_MASK: u8 = 0b0000_0111;

        let mut apu = Apu::new();

        let frequency: u16 = 1024;
        let expected_frequency = frequency + (frequency >> 1);

        apu.write(NR10_ADDRESS, 0b0001_0001);
        apu.write(NR12_ADDRESS, NR12_DAC_MASK);
        apu.write(NR13_ADDRESS, frequency as u8);
        apu.write(
            NR14_ADDRESS,
            ((frequency >> u8::BITS) as u8 & NR14_FREQUENCY_MASK) | NR14_TRIGGER_MASK,
        );

        assert_eq!(apu.channel1.get_frequency_for_test(), frequency);

        for _ in 0..FRAME_SEQUENCER_PERIOD * 2 {
            apu.tick();
        }

        assert_eq!(apu.channel1.get_frequency_for_test(), expected_frequency);
    }

    #[test]
    fn nr51_routes_channel1_to_left_output() {
        let mut apu = Apu::new();

        setup_channel1(&mut apu);

        apu.write(NR51_ADDRESS, 0b0001_0000);

        let (left, right) = apu.get_mixed_output();

        assert!(left > 0);
        assert_eq!(right, 0);
    }

    #[test]
    fn nr51_routes_channel1_to_right_output() {
        let mut apu = Apu::new();

        setup_channel1(&mut apu);

        apu.write(NR51_ADDRESS, 0b0000_0001);

        let (left, right) = apu.get_mixed_output();

        assert_eq!(left, 0);
        assert!(right > 0);
    }

    #[test]
    fn nr51_routes_channel1_to_both_outputs() {
        let mut apu = Apu::new();

        setup_channel1(&mut apu);

        apu.write(NR51_ADDRESS, 0b0001_0001);

        let (left, right) = apu.get_mixed_output();

        assert!(left > 0);
        assert!(right > 0);
    }

    #[test]
    fn nr51_can_mute_channel1_from_both_outputs() {
        let mut apu = Apu::new();

        setup_channel1(&mut apu);

        apu.write(NR51_ADDRESS, 0);

        let (left, right) = apu.get_mixed_output();

        assert_eq!(left, 0);
        assert_eq!(right, 0);
    }

    #[test]
    fn nr51_routes_all_channels_independently() {
        const NR51_ROUTE_CHANNELS_1_AND_3_RIGHT_2_AND_4_LEFT: u8 = 0b1010_0101;
        const EXPECTED_LEFT_OUTPUT: u16 = 30;
        const EXPECTED_RIGHT_OUTPUT: u16 = 18;

        let mut apu = Apu::new();

        setup_channel1(&mut apu);
        setup_channel2(&mut apu);
        setup_channel3(&mut apu);
        setup_channel4(&mut apu);

        apu.write(NR50_ADDRESS, 0);
        apu.write(NR51_ADDRESS, NR51_ROUTE_CHANNELS_1_AND_3_RIGHT_2_AND_4_LEFT);

        let (left, right) = apu.get_mixed_output();

        assert_eq!(left, EXPECTED_LEFT_OUTPUT);
        assert_eq!(right, EXPECTED_RIGHT_OUTPUT);
    }

    #[test]
    fn nr50_applies_left_volume_independently() {
        const NR50_LEFT_VOLUME_MAX: u8 = 0b0111_0000;
        const NR51_ROUTE_CHANNEL2_LEFT: u8 = 0b0010_0000;
        const EXPECTED_LEFT_OUTPUT: u16 = 120;
        const EXPECTED_RIGHT_OUTPUT: u16 = 0;

        let mut apu = Apu::new();

        setup_channel2(&mut apu);
        apu.write(NR50_ADDRESS, NR50_LEFT_VOLUME_MAX);
        apu.write(NR51_ADDRESS, NR51_ROUTE_CHANNEL2_LEFT);

        let (left, right) = apu.get_mixed_output();

        assert_eq!(left, EXPECTED_LEFT_OUTPUT);
        assert_eq!(right, EXPECTED_RIGHT_OUTPUT);
    }

    #[test]
    fn nr50_applies_right_volume_independently() {
        const NR50_RIGHT_VOLUME_MAX: u8 = 0b0000_0111;
        const NR51_ROUTE_CHANNEL1_RIGHT: u8 = 0b0000_0001;
        const EXPECTED_LEFT_OUTPUT: u16 = 0;
        const EXPECTED_RIGHT_OUTPUT: u16 = 120;

        let mut apu = Apu::new();

        setup_channel1(&mut apu);
        apu.write(NR50_ADDRESS, NR50_RIGHT_VOLUME_MAX);
        apu.write(NR51_ADDRESS, NR51_ROUTE_CHANNEL1_RIGHT);

        let (left, right) = apu.get_mixed_output();

        assert_eq!(left, EXPECTED_LEFT_OUTPUT);
        assert_eq!(right, EXPECTED_RIGHT_OUTPUT);
    }

    #[test]
    fn nr50_minimum_volume_preserves_channel_output() {
        const NR50_MIN_VOLUME: u8 = 0;
        const NR51_ROUTE_CHANNEL1_RIGHT: u8 = 0b0000_0001;
        const EXPECTED_OUTPUT: u16 = 15;

        let mut apu = Apu::new();

        setup_channel1(&mut apu);
        apu.write(NR50_ADDRESS, NR50_MIN_VOLUME);
        apu.write(NR51_ADDRESS, NR51_ROUTE_CHANNEL1_RIGHT);

        let (_, right) = apu.get_mixed_output();

        assert_eq!(right, EXPECTED_OUTPUT);
    }

    #[test]
    fn nr50_applies_different_volumes_to_left_and_right() {
        const NR50_LEFT_VOLUME_MAX_RIGHT_VOLUME_MIN: u8 = 0b0111_0000;
        const NR51_ROUTE_CHANNEL1_TO_BOTH: u8 = 0b0001_0001;
        const EXPECTED_LEFT_OUTPUT: u16 = 120;
        const EXPECTED_RIGHT_OUTPUT: u16 = 15;

        let mut apu = Apu::new();

        setup_channel1(&mut apu);
        apu.write(NR50_ADDRESS, NR50_LEFT_VOLUME_MAX_RIGHT_VOLUME_MIN);
        apu.write(NR51_ADDRESS, NR51_ROUTE_CHANNEL1_TO_BOTH);

        let (left, right) = apu.get_mixed_output();

        assert_eq!(left, EXPECTED_LEFT_OUTPUT);
        assert_eq!(right, EXPECTED_RIGHT_OUTPUT);
    }

    #[test]
    fn nr50_and_nr51_read_zero_when_apu_is_disabled() {
        const NR52_ADDRESS: u16 = 0xFF26;
        const NR50_ADDRESS: u16 = 0xFF24;
        const NR51_ADDRESS: u16 = 0xFF25;
        const APU_DISABLE: u8 = 0x00;
        const EXPECTED_VALUE: u8 = 0x00;

        let mut apu = Apu::new();

        apu.write(NR52_ADDRESS, APU_DISABLE);

        assert_eq!(apu.read(NR50_ADDRESS), EXPECTED_VALUE);
        assert_eq!(apu.read(NR51_ADDRESS), EXPECTED_VALUE);
    }

    #[test]
    fn nr50_and_nr51_writes_are_ignored_when_apu_is_disabled() {
        const NR52_ADDRESS: u16 = 0xFF26;
        const NR50_ADDRESS: u16 = 0xFF24;
        const NR51_ADDRESS: u16 = 0xFF25;
        const NR50_WRITE_VALUE: u8 = 0b0111_0111;
        const NR51_WRITE_VALUE: u8 = 0b1111_1111;
        const APU_DISABLE: u8 = 0x00;
        const EXPECTED_VALUE: u8 = 0x00;

        let mut apu = Apu::new();

        apu.write(NR52_ADDRESS, APU_DISABLE);
        apu.write(NR50_ADDRESS, NR50_WRITE_VALUE);
        apu.write(NR51_ADDRESS, NR51_WRITE_VALUE);

        assert_eq!(apu.read(NR50_ADDRESS), EXPECTED_VALUE);
        assert_eq!(apu.read(NR51_ADDRESS), EXPECTED_VALUE);
    }

    #[test]
    fn disabling_apu_resets_all_channels() {
        const NR52_ADDRESS: u16 = 0xFF26;
        const APU_DISABLE: u8 = 0x00;
        const ACTIVE_CHANNELS_MASK: u8 = 0b0000_1111;
        const EXPECTED_ACTIVE_CHANNELS: u8 = ACTIVE_CHANNELS_MASK;
        const EXPECTED_DISABLED_STATUS: u8 = 0b0111_0000;

        let mut apu = Apu::new();

        setup_channel1(&mut apu);
        setup_channel2(&mut apu);
        setup_channel3(&mut apu);
        setup_channel4(&mut apu);

        assert_eq!(
            apu.read(NR52_ADDRESS) & ACTIVE_CHANNELS_MASK,
            EXPECTED_ACTIVE_CHANNELS
        );

        apu.write(NR52_ADDRESS, APU_DISABLE);

        assert_eq!(apu.read(NR52_ADDRESS), EXPECTED_DISABLED_STATUS);
    }

    #[test]
    fn channel_triggers_are_ignored_when_apu_is_disabled() {
        const NR52_ADDRESS: u16 = 0xFF26;
        const NR12_ADDRESS: u16 = 0xFF12;
        const NR14_ADDRESS: u16 = 0xFF14;
        const NR22_ADDRESS: u16 = 0xFF17;
        const NR24_ADDRESS: u16 = 0xFF19;
        const NR30_ADDRESS: u16 = 0xFF1A;
        const NR34_ADDRESS: u16 = 0xFF1E;
        const NR42_ADDRESS: u16 = 0xFF21;
        const NR44_ADDRESS: u16 = 0xFF23;

        const APU_DISABLE: u8 = 0x00;
        const DAC_ENABLE_AND_MAX_VOLUME: u8 = 0xF0;
        const DAC_ENABLE: u8 = 0x80;
        const TRIGGER: u8 = 0x80;
        const EXPECTED_DISABLED_STATUS: u8 = 0b0111_0000;

        let mut apu = Apu::new();

        apu.write(NR52_ADDRESS, APU_DISABLE);

        apu.write(NR12_ADDRESS, DAC_ENABLE_AND_MAX_VOLUME);
        apu.write(NR14_ADDRESS, TRIGGER);

        apu.write(NR22_ADDRESS, DAC_ENABLE_AND_MAX_VOLUME);
        apu.write(NR24_ADDRESS, TRIGGER);

        apu.write(NR30_ADDRESS, DAC_ENABLE);
        apu.write(NR34_ADDRESS, TRIGGER);

        apu.write(NR42_ADDRESS, DAC_ENABLE_AND_MAX_VOLUME);
        apu.write(NR44_ADDRESS, TRIGGER);

        assert_eq!(apu.read(NR52_ADDRESS), EXPECTED_DISABLED_STATUS);
    }

    #[test]
    fn enabling_apu_does_not_activate_channels_automatically() {
        const NR52_ADDRESS: u16 = 0xFF26;
        const APU_DISABLE: u8 = 0x00;
        const APU_ENABLE: u8 = 0x80;
        const EXPECTED_ENABLED_STATUS: u8 = 0b1111_0000;

        let mut apu = Apu::new();

        apu.write(NR52_ADDRESS, APU_DISABLE);
        apu.write(NR52_ADDRESS, APU_ENABLE);

        assert_eq!(apu.read(NR52_ADDRESS), EXPECTED_ENABLED_STATUS);
    }

    #[test]
    fn length_register_writes_are_allowed_when_apu_is_disabled() {
        const NR52_ADDRESS: u16 = 0xFF26;
        const NR11_ADDRESS: u16 = 0xFF11;
        const NR21_ADDRESS: u16 = 0xFF16;
        const NR31_ADDRESS: u16 = 0xFF1B;
        const NR41_ADDRESS: u16 = 0xFF20;

        const APU_DISABLE: u8 = 0x00;
        const LENGTH_REGISTER_VALUE: u8 = 0x01;

        let mut apu = Apu::new();

        apu.write(NR52_ADDRESS, APU_DISABLE);

        apu.write(NR11_ADDRESS, LENGTH_REGISTER_VALUE);
        apu.write(NR21_ADDRESS, LENGTH_REGISTER_VALUE);
        apu.write(NR31_ADDRESS, LENGTH_REGISTER_VALUE);
        apu.write(NR41_ADDRESS, LENGTH_REGISTER_VALUE);

        const EXPECTED_DISABLED_STATUS: u8 = 0x70;
        assert_eq!(apu.read(NR52_ADDRESS), EXPECTED_DISABLED_STATUS);
    }

    #[test]
    fn disabling_apu_preserves_wave_ram() {
        const WAVE_RAM_ADDRESS: u16 = 0xFF30;
        const NR52_ADDRESS: u16 = 0xFF26;
        const WAVE_RAM_VALUE: u8 = 0xA5;
        const APU_DISABLE: u8 = 0x00;

        let mut apu = Apu::new();

        apu.write(WAVE_RAM_ADDRESS, WAVE_RAM_VALUE);
        apu.write(NR52_ADDRESS, APU_DISABLE);

        assert_eq!(apu.read(WAVE_RAM_ADDRESS), WAVE_RAM_VALUE);
    }

    #[test]
    fn disabling_and_reenabling_apu_resets_channel_registers() {
        const NR52_ADDRESS: u16 = 0xFF26;
        const NR12_ADDRESS: u16 = 0xFF12;
        const NR22_ADDRESS: u16 = 0xFF17;
        const NR30_ADDRESS: u16 = 0xFF1A;
        const NR42_ADDRESS: u16 = 0xFF21;

        const APU_DISABLE: u8 = 0x00;
        const APU_ENABLE: u8 = 0x80;
        const DAC_ENABLE_AND_MAX_VOLUME: u8 = 0xF0;
        const DAC_ENABLE: u8 = 0x80;

        const EXPECTED_NR12: u8 = 0x00;
        const EXPECTED_NR22: u8 = 0x00;
        const EXPECTED_NR30: u8 = 0x7F;
        const EXPECTED_NR42: u8 = 0x00;

        let mut apu = Apu::new();

        apu.write(NR12_ADDRESS, DAC_ENABLE_AND_MAX_VOLUME);
        apu.write(NR22_ADDRESS, DAC_ENABLE_AND_MAX_VOLUME);
        apu.write(NR30_ADDRESS, DAC_ENABLE);
        apu.write(NR42_ADDRESS, DAC_ENABLE_AND_MAX_VOLUME);

        apu.write(NR52_ADDRESS, APU_DISABLE);
        apu.write(NR52_ADDRESS, APU_ENABLE);

        assert_eq!(apu.read(NR12_ADDRESS), EXPECTED_NR12);
        assert_eq!(apu.read(NR22_ADDRESS), EXPECTED_NR22);
        assert_eq!(apu.read(NR30_ADDRESS), EXPECTED_NR30);
        assert_eq!(apu.read(NR42_ADDRESS), EXPECTED_NR42);
    }

    #[test]
    fn disabling_and_reenabling_apu_resets_nr50_and_nr51() {
        const NR50_ADDRESS: u16 = 0xFF24;
        const NR51_ADDRESS: u16 = 0xFF25;
        const NR52_ADDRESS: u16 = 0xFF26;
        const APU_DISABLE: u8 = 0x00;
        const APU_ENABLE: u8 = 0x80;
        const REGISTER_VALUE: u8 = 0xFF;
        const EXPECTED_REGISTER_VALUE: u8 = 0x00;

        let mut apu = Apu::new();

        apu.write(NR50_ADDRESS, REGISTER_VALUE);
        apu.write(NR51_ADDRESS, REGISTER_VALUE);

        apu.write(NR52_ADDRESS, APU_DISABLE);
        apu.write(NR52_ADDRESS, APU_ENABLE);

        assert_eq!(apu.read(NR50_ADDRESS), EXPECTED_REGISTER_VALUE);
        assert_eq!(apu.read(NR51_ADDRESS), EXPECTED_REGISTER_VALUE);
    }

    #[test]
    fn generates_expected_number_of_samples() {
        const EXPECTED_SAMPLES: usize = 44_100;

        let mut apu = Apu::new();

        for _ in 0..APU_CLOCK_HZ {
            apu.tick();
        }

        assert_eq!(apu.take_samples().len(), EXPECTED_SAMPLES);
    }

    #[test]
    fn generates_silence_when_apu_is_disabled() {
        const NR52_ADDRESS: u16 = 0xFF26;
        const APU_DISABLE: u8 = 0x00;
        const T_CYCLES: u32 = 1_000;

        let mut apu = Apu::new();
        apu.write(NR52_ADDRESS, APU_DISABLE);

        for _ in 0..T_CYCLES {
            apu.tick();
        }

        let samples = apu.take_samples();

        assert!(!samples.is_empty());
        assert!(
            samples
                .iter()
                .all(|&(left, right)| left == 0.0 && right == 0.0)
        );
    }
}
