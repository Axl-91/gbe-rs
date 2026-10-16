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

use crate::apu::{channel2::Channel2, channel3::Channel3, channel4::Channel4};

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
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            CHANNEL1_REGISTERS_START..=CHANNEL1_REGISTERS_END => self.channel1.read(address),
            CHANNEL2_REGISTERS_START..=CHANNEL2_REGISTERS_END => self.channel2.read(address),
            CHANNEL3_REGISTERS_START..=CHANNEL3_REGISTERS_END => self.channel3.read(address),
            CHANNEL4_REGISTERS_START..=CHANNEL4_REGISTERS_END => self.channel4.read(address),
            NR50_ADDRESS => self.nr50,
            NR51_ADDRESS => self.nr51,
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
            CHANNEL2_REGISTERS_START..=CHANNEL2_REGISTERS_END if self.enabled => {
                let extra_length_clock = self.frame_sequencer.next_step_skips_length_clock();
                self.channel2
                    .write_with_extra_length_clock(address, value, extra_length_clock);
            }
            CHANNEL3_REGISTERS_START..=CHANNEL3_REGISTERS_END if self.enabled => {
                let extra_length_clock = self.frame_sequencer.next_step_skips_length_clock();

                self.channel3
                    .write_with_extra_length_clock(address, value, extra_length_clock);
            }
            CHANNEL4_REGISTERS_START..=CHANNEL4_REGISTERS_END => {
                let extra_length_clock = self.frame_sequencer.next_step_skips_length_clock();

                self.channel4
                    .write_with_extra_length_clock(address, value, extra_length_clock);
            }
            WAVE_RAM_START..=WAVE_RAM_END if self.enabled => {
                self.channel3.write(address, value);
            }
            NR50_ADDRESS => self.nr50 = value,
            NR51_ADDRESS => self.nr51 = value,
            NR52_ADDRESS => {
                let should_enable = value & APU_ENABLE_MASK != 0;

                if self.enabled && !should_enable {
                    self.channel1 = Channel1::new();
                    self.channel2 = Channel2::new();
                    self.channel3 = Channel3::new();
                    self.channel4 = Channel4::new();
                    self.frame_sequencer = FrameSequencer::new();
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
        if !self.enabled {
            return;
        }

        self.channel1.tick();
        self.channel2.tick();
        self.channel3.tick();
        self.channel4.tick();

        if self.frame_sequencer.tick() {
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
}

impl Default for Apu {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
