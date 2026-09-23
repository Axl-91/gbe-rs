//! Emulation of the OAM DMA (Direct Memory Access) controller.
//!
//! On real hardware, writing to the `FF46` (DMA) register triggers a
//! transfer that copies 160 bytes (`0x00`..=`0x9F`) from `source << 8`
//! into OAM, at a rate of one byte every 4 T-Cycles, after an initial
//! startup delay before the transfer actually begins.
//!
//! This module models that state machine cycle by cycle, including the
//! case where a DMA is retriggered while a previous one is still in
//! progress (a restart), which on real hardware resets the timing with
//! a different delay (`RESTART_DELAY`).

/// Last valid index within an OAM transfer block (160 bytes: 0x00..=0x9F).
const LAST_INDEX: u8 = 0x9F;

// Delays expressed in T-Cycles
const START_DELAY: u8 = 8;
const TRANSFER_DELAY: u8 = 4;
const RESTART_DELAY: u8 = 8;

#[cfg(test)]
mod tests;

/// Possible states of the DMA state machine.
#[derive(Debug)]
enum DmaState {
    Inactive,
    Starting,
    Transferring { index: u8 },
}

/// OAM DMA controller.
///
/// [`Dma::tick`] must be called once per T-Cycle to advance the state
/// machine, and [`Dma::consume_cycle`] to decrement the various delay
/// counters within that same cycle.
#[derive(Debug)]
pub struct Dma {
    state: DmaState,
    active_source: u16,
    pub pending_source: Option<u16>,

    start_delay: u8,
    transfer_delay: u8,
    restart_delay: u8,
}

impl Dma {
    pub fn new() -> Self {
        Self {
            state: DmaState::Inactive,
            active_source: 0,
            pending_source: None,

            start_delay: 0,
            transfer_delay: 0,
            restart_delay: 0,
        }
    }

    pub fn get_index(&self) -> u8 {
        match self.state {
            DmaState::Inactive => 0,
            DmaState::Starting => 0,
            DmaState::Transferring { index } => index,
        }
    }

    fn is_starting(&self) -> bool {
        matches!(self.state, DmaState::Starting)
    }

    pub fn is_transferring(&self) -> bool {
        matches!(self.state, DmaState::Transferring { .. })
    }

    /// Requests the start of a DMA transfer from page `value` (the
    /// actual source address is `value << 8`).
    ///
    /// If a transfer is already in progress or starting, the new source
    /// is stored as pending (`pending_source`) and will be applied after
    /// `RESTART_DELAY` cycles. If no transfer is currently active, the
    /// `Starting` phase begins immediately with a `START_DELAY` cycle
    /// countdown.
    pub fn start(&mut self, value: u8) {
        let source = (value as u16) << 8;

        if self.is_transferring() || self.is_starting() {
            self.pending_source = Some(source);
            self.restart_delay = RESTART_DELAY;
        } else {
            self.active_source = source;
            self.state = DmaState::Starting;
            self.start_delay = START_DELAY;
        }
    }

    pub fn source_address(&self) -> u16 {
        match self.state {
            DmaState::Inactive => 0,
            DmaState::Starting => 0,
            DmaState::Transferring { index } => self.active_source + index as u16,
        }
    }

    pub fn can_transfer_byte(&self) -> bool {
        self.is_transferring() && self.transfer_delay == 0
    }

    fn set_new_source(&mut self, new_source: u16) {
        self.active_source = new_source;
        self.pending_source = None;
        self.state = DmaState::Transferring { index: 0 };
        self.transfer_delay = TRANSFER_DELAY;
    }

    /// Decrements by one cycle whichever delay counters are currently
    /// active (`start_delay`, `transfer_delay`, `restart_delay`),
    /// depending on the current state. Must be called once per T-Cycle,
    /// alongside [`Dma::tick`].
    pub fn consume_cycle(&mut self) {
        if self.is_starting() {
            self.start_delay -= 1
        }
        if self.is_transferring() {
            self.transfer_delay -= 1;
        }
        if self.pending_source.is_some() {
            self.restart_delay -= 1;
        }
    }

    /// Advances the DMA state machine by one T-Cycle.
    ///
    /// Priority order:
    /// 1. If there is a pending source (`pending_source`) and
    ///    `restart_delay` has already reached 0, it is applied
    ///    immediately (restarting the transfer), and nothing else is
    ///    processed this cycle.
    /// 2. If in `Starting` and `start_delay` has reached 0, transitions
    ///    to `Transferring { index: 0 }`.
    /// 3. If `Transferring` and a byte should be transferred this cycle
    ///    ([`Dma::can_transfer_byte`]), advances to the next index, or
    ///    returns to `Inactive` if the last byte (`LAST_INDEX`) has
    ///    already been transferred.
    pub fn tick(&mut self) {
        if let Some(new_source) = self.pending_source
            && self.restart_delay == 0
        {
            self.set_new_source(new_source);
            return;
        }
        match self.state {
            DmaState::Starting => {
                if self.start_delay == 0 {
                    self.state = DmaState::Transferring { index: 0 };
                    self.transfer_delay = TRANSFER_DELAY;
                }
            }
            DmaState::Transferring { index } if self.can_transfer_byte() => {
                if index == LAST_INDEX {
                    self.state = DmaState::Inactive;
                    self.transfer_delay = 0;
                } else {
                    self.state = DmaState::Transferring { index: index + 1 };
                    self.transfer_delay = TRANSFER_DELAY;
                }
            }

            _ => {}
        }
    }
}

impl Default for Dma {
    fn default() -> Self {
        Self::new()
    }
}
