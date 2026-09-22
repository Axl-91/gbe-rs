const LAST_INDEX: u8 = 0x9F;

// Delays expressed in T-Cycles
const START_DELAY: u8 = 8;
const TRANSFER_DELAY: u8 = 4;
const RESTART_DELAY: u8 = 8;

#[cfg(test)]
mod tests;

#[derive(Debug)]
enum DmaState {
    Inactive,
    Starting,
    Transferring { index: u8 },
}

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

    fn set_new_source(&mut self, new_source: u16) {
        self.active_source = new_source;
        self.pending_source = None;
        self.state = DmaState::Transferring { index: 0 };
        self.transfer_delay = TRANSFER_DELAY;
    }

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
