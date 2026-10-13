//! Game Boy Audio Processing Unit (APU) implementation.
//!
//! Generates audio by managing the sound channels and their timing.
//! Coordinates channel updates through the frame sequencer and provides
//! the interface for audio register access.

mod frame_sequencer;

use frame_sequencer::FrameSequencer;

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
