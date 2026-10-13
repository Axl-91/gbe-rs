const FRAME_SEQUENCER_PERIOD: u16 = 8192;
const FRAME_SEQUENCER_STEPS: u8 = 8;

pub(super) struct FrameSequencer {
    cycles: u16,
    step: u8,
}

impl FrameSequencer {
    /// Creates a frame sequencer at the beginning of its cycle.
    pub fn new() -> Self {
        Self { cycles: 0, step: 0 }
    }

    pub fn get_step(&self) -> u8 {
        self.step
    }

    /// Advances the sequencer by one T-cycle.
    ///
    /// Returns `true` when a frame-sequencer step has elapsed.
    pub fn tick(&mut self) -> bool {
        self.cycles += 1;

        if self.cycles == FRAME_SEQUENCER_PERIOD {
            self.cycles = 0;
            self.step = (self.step + 1) % FRAME_SEQUENCER_STEPS;
            return true;
        }

        false
    }
}
