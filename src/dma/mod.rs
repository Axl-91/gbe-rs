const LAST_INDEX: u8 = 0x9F;
const TRANSFER_CYCLES: u8 = 4;

#[cfg(test)]
mod tests;

enum DmaState {
    Inactive,
    Transferring { index: u8 },
}

pub struct Dma {
    state: DmaState,
    source: u16,
    transfer_cycles: u8,
}

impl Dma {
    pub fn new() -> Self {
        Self {
            state: DmaState::Inactive,
            source: 0,
            transfer_cycles: 0,
        }
    }

    pub fn get_index(&self) -> u8 {
        match self.state {
            DmaState::Inactive => 0,
            DmaState::Transferring { index } => index,
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self.state, DmaState::Transferring { .. })
    }

    pub fn start(&mut self, value: u8) {
        self.source = (value as u16) << 8;
        self.state = DmaState::Transferring { index: 0 };
        self.transfer_cycles = TRANSFER_CYCLES;
    }

    pub fn source_address(&self) -> u16 {
        match self.state {
            DmaState::Inactive => 0,
            DmaState::Transferring { index } => self.source + index as u16,
        }
    }

    pub fn is_ready_to_transfer(&self) -> bool {
        self.transfer_cycles == 0
    }

    pub fn consume_cycle(&mut self) {
        self.transfer_cycles -= 1;
    }

    pub fn tick(&mut self) {
        match self.state {
            DmaState::Inactive => {}

            DmaState::Transferring { index } => {
                if index == LAST_INDEX {
                    self.state = DmaState::Inactive;
                    self.transfer_cycles = 0;
                } else {
                    self.state = DmaState::Transferring { index: index + 1 };
                    self.transfer_cycles = TRANSFER_CYCLES;
                }
            }
        }
    }
}

impl Default for Dma {
    fn default() -> Self {
        Self::new()
    }
}
