//! Game Boy Audio Processing Unit (APU) implementation.
//!
//! Generates audio by managing the sound channels and their timing.
//! Coordinates channel updates through the frame sequencer and provides
//! the interface for audio register access.

mod channel1;
mod frame_sequencer;

use channel1::Channel1;
use frame_sequencer::FrameSequencer;

const NR52_ADDRESS: u16 = 0xFF26;
const NR52_UNUSED_BITS: u8 = 0b0111_0000;
const APU_ENABLE_MASK: u8 = 0b1000_0000;

const CHANNEL1_REGISTERS_START: u16 = 0xFF10;
const CHANNEL1_REGISTERS_END: u16 = 0xFF14;

/// Game Boy Audio Processing Unit.
pub struct Apu {
    enabled: bool,
    frame_sequencer: FrameSequencer,
    channel1: Channel1,
}

impl Apu {
    pub fn new() -> Self {
        Self {
            enabled: true,
            frame_sequencer: FrameSequencer::new(),
            channel1: Channel1::new(),
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            CHANNEL1_REGISTERS_START..=CHANNEL1_REGISTERS_END => self.channel1.read(address),
            NR52_ADDRESS => {
                let enabled = if self.enabled { APU_ENABLE_MASK } else { 0 };

                enabled | NR52_UNUSED_BITS | self.channel_status()
            }
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            CHANNEL1_REGISTERS_START..=CHANNEL1_REGISTERS_END if self.enabled => {
                self.channel1.write(address, value);
            }
            NR52_ADDRESS => {
                let should_enable = value & APU_ENABLE_MASK != 0;

                if self.enabled && !should_enable {
                    self.channel1 = Channel1::new();
                    self.frame_sequencer = FrameSequencer::new();
                }

                self.enabled = should_enable;
            }
            _ => {}
        }
    }

    fn channel_status(&self) -> u8 {
        u8::from(self.channel1.is_active())
    }

    /// Advances the APU by one T-cycle.
    pub fn tick(&mut self) {
        if !self.enabled {
            return;
        }

        self.frame_sequencer.tick();
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
    fn reading_unimplemented_apu_register_returns_unmapped_value() {
        let apu = Apu::new();

        assert_eq!(apu.read(NR52_ADDRESS - 1), 0xFF);
    }

    #[test]
    fn writing_unimplemented_apu_register_does_not_change_nr52() {
        let mut apu = Apu::new();

        apu.write(NR52_ADDRESS - 1, 0);

        assert_eq!(apu.read(NR52_ADDRESS), APU_ENABLE_MASK | NR52_UNUSED_BITS);
    }
}
