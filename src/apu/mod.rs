//! Game Boy Audio Processing Unit (APU) implementation.
//!
//! Generates audio by managing the sound channels and their timing.
//! Coordinates channel updates through the frame sequencer and provides
//! the interface for audio register access.

mod frame_sequencer;

use frame_sequencer::FrameSequencer;

const NR52_ADDRESS: u16 = 0xFF26;
const NR52_UNUSED_BITS: u8 = 0b0111_0000;
const NR52_CHANNEL_STATUS_MASK: u8 = 0b0000_1111;
const APU_ENABLE_MASK: u8 = 0b1000_0000;

/// Game Boy Audio Processing Unit.
pub struct Apu {
    enabled: bool,
    frame_sequencer: FrameSequencer,
}

impl Apu {
    pub fn new() -> Self {
        Self {
            enabled: true,
            frame_sequencer: FrameSequencer::new(),
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            NR52_ADDRESS => {
                let enabled = if self.enabled { APU_ENABLE_MASK } else { 0 };

                enabled | NR52_UNUSED_BITS | (self.channel_status() & NR52_CHANNEL_STATUS_MASK)
            }
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            NR52_ADDRESS => {
                self.enabled = value & APU_ENABLE_MASK != 0;

                if !self.enabled {
                    self.frame_sequencer = FrameSequencer::new();
                }
            }
            _ => {}
        }
    }

    fn channel_status(&self) -> u8 {
        0
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
