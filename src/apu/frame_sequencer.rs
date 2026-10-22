const FRAME_SEQUENCER_PERIOD: u16 = 8192;
const FRAME_SEQUENCER_STEPS: u8 = 8;

/// Step stored on power-on. The first tick after power-on advances to
/// step 0, which clocks the length counters.
const POWER_ON_STEP: u8 = FRAME_SEQUENCER_STEPS - 1;

pub(super) struct FrameSequencer {
    cycles: u16,
    step: u8,
}

impl FrameSequencer {
    /// Creates a frame sequencer at the beginning of its cycle.
    pub fn new() -> Self {
        Self { cycles: 0, step: 0 }
    }

    /// Resets the step on APU power-on.
    ///
    /// The cycle counter is left untouched: on hardware it is driven by the
    /// system divider, so the phase is not affected by the APU power state.
    pub fn power_on(&mut self) {
        self.step = POWER_ON_STEP;
    }

    pub fn get_step(&self) -> u8 {
        self.step
    }

    pub(super) fn next_step_skips_length_clock(&self) -> bool {
        let next_step = (self.step + 1) % FRAME_SEQUENCER_STEPS;
        !next_step.is_multiple_of(2)
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
