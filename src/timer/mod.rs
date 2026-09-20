//! Game Boy timer (DIV, TIMA, TMA and TAC registers).
//!
//! The timer is driven by a 16-bit internal counter that increments once per
//! T-cycle. DIV is the upper byte of that counter. TIMA increments on the
//! falling edge of one bit of the counter, selected by TAC and gated by the
//! TAC enable bit. Because of that, writing TIMA never affects the phase of
//! the counter, which is what the cycle-accurate timing tests rely on.

const DIV_ADDRESS: u16 = 0xFF04;
const TIMA_ADDRESS: u16 = 0xFF05;
const TMA_ADDRESS: u16 = 0xFF06;
const TAC_ADDRESS: u16 = 0xFF07;

const TAC_ENABLE_MASK: u8 = 0b100;
const TAC_UNUSED_BITS: u8 = 0xF8;

/// T-cycles between TIMA overflowing and TMA being loaded into it.
const RELOAD_DELAY_T_CYCLES: u8 = 4;

#[cfg(test)]
mod tests;

pub struct Timer {
    /// Internal 16-bit counter, incremented once per T-cycle. DIV is its upper byte.
    sys_counter: u16,
    tima: u8,
    tma: u8,
    tac: u8,
    /// T-cycles left until TMA is loaded into TIMA after an overflow (0 = none pending).
    reload_delay: u8,
    /// T-cycles left in the window where TIMA has just been loaded from TMA.
    reload_window: u8,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            sys_counter: 0x00AB, // DMG value after the boot ROM; use 0 if you run the boot ROM
            tima: 0,
            tma: 0,
            tac: 0xF8,
            reload_delay: 0,
            reload_window: 0,
        }
    }

    /// Bit of the internal counter selected by TAC bits 0-1.
    fn selected_bit(&self) -> u16 {
        match self.tac & 0b11 {
            0 => 9, // 4096 Hz
            1 => 3, // 262144 Hz (one TIMA increment every 16 T-cycles)
            2 => 5, // 65536 Hz
            _ => 7, // 16384 Hz
        }
    }

    // Check if TAC is enabled (bit 2 of tac) and the selected clock bit is 1
    fn tac_non_zero(&self) -> bool {
        self.tac & TAC_ENABLE_MASK != 0 && (self.sys_counter >> self.selected_bit()) & 0x0001 != 0
    }

    fn increment_tima(&mut self) {
        let (value, overflowed) = self.tima.overflowing_add(1);
        self.tima = value; // reads as 0x00 until the reload happens
        if overflowed {
            self.reload_delay = RELOAD_DELAY_T_CYCLES;
        }
    }

    /// Advances the timer by one T-cycle.
    /// Returns true when the timer interrupt (IF bit 2) must be requested.
    pub fn tick(&mut self) -> bool {
        let mut interrupt_requested = false;

        self.reload_window = self.reload_window.saturating_sub(1);

        if self.reload_delay > 0 {
            self.reload_delay -= 1;
            if self.reload_delay == 0 {
                self.tima = self.tma;
                self.reload_window = RELOAD_DELAY_T_CYCLES;
                interrupt_requested = true;
            }
        }

        let tac_non_zero_before = self.tac_non_zero();
        self.sys_counter = self.sys_counter.wrapping_add(1);
        if tac_non_zero_before && !self.tac_non_zero() {
            self.increment_tima();
        }
        interrupt_requested
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            DIV_ADDRESS => (self.sys_counter >> 8) as u8,
            TIMA_ADDRESS => self.tima,
            TMA_ADDRESS => self.tma,
            TAC_ADDRESS => self.tac | TAC_UNUSED_BITS,
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            DIV_ADDRESS => {
                // Resetting the counter can produce a falling edge.
                if self.tac_non_zero() {
                    self.increment_tima();
                }
                self.sys_counter = 0;
            }
            TIMA_ADDRESS => {
                // Ignored while TMA is being loaded into TIMA.
                if self.reload_window == 0 {
                    self.tima = value;
                    self.reload_delay = 0; // cancels the pending reload and interrupt
                }
            }
            TMA_ADDRESS => {
                self.tma = value;
                if self.reload_window > 0 {
                    self.tima = value;
                }
            }
            TAC_ADDRESS => {
                let tac_non_zero_before = self.tac_non_zero();
                self.tac = value & 0b111;
                if tac_non_zero_before && !self.tac_non_zero() {
                    self.increment_tima();
                }
            }
            _ => {}
        }
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}
