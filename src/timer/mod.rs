//! Game Boy timer.
//!
//! The timer is driven by the internal divider and is controlled through
//! the TAC register. TIMA is incremented on the falling edge of the
//! selected divider bit while the timer is enabled.
//!
//! Timer registers:
//!
//! - `FF04`: DIV — upper 8 bits of the internal divider.
//! - `FF05`: TIMA — timer counter.
//! - `FF06`: TMA — timer modulo.
//! - `FF07`: TAC — timer control and frequency selection.

#[cfg(test)]
mod tests;

pub struct Timer {
    div: u16,
    tima: u8,
    tma: u8,
    tac: u8,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            div: 0,
            tima: 0,
            tma: 0,
            tac: 0,
        }
    }

    pub(crate) fn read_div(&self) -> u8 {
        (self.div >> 8) as u8
    }

    pub(crate) fn read_tima(&self) -> u8 {
        self.tima
    }

    pub(crate) fn read_tma(&self) -> u8 {
        self.tma
    }

    pub(crate) fn read_tac(&self) -> u8 {
        self.tac
    }

    pub(crate) fn reset_div(&mut self) {
        let timer_enabled = self.tac & (1 << 2) != 0;
        let div_bit = self.get_frequency_bit();

        let previous_bit = ((self.div >> div_bit) & 0x01) as u8;
        self.div = 0;

        if timer_enabled && previous_bit == 1 {
            self.increment_tima();
        }
    }

    pub(crate) fn write_tima(&mut self, value: u8) {
        self.tima = value;
    }

    pub(crate) fn write_tma(&mut self, value: u8) {
        self.tma = value;
    }

    pub(crate) fn write_tac(&mut self, value: u8) {
        self.tac = value;
    }

    /// Returns the divider bit selected by the timer frequency.
    ///
    /// The two least significant bits of `TAC` select which bit of the
    /// internal divider drives the timer:
    ///
    /// - `00` → bit 9
    /// - `01` → bit 3
    /// - `10` → bit 5
    /// - `11` → bit 7
    fn get_frequency_bit(&self) -> u8 {
        let frequency = self.tac & 0b11;
        match frequency {
            0b00 => 9,
            0b01 => 3,
            0b10 => 5,
            0b11 => 7,
            _ => unreachable!(),
        }
    }
    fn increment_tima(&mut self) {
        if self.tima == 0xFF {
            self.tima = self.tma;
        } else {
            self.tima += 1;
        }
    }

    /// Advances the timer by one T-Cycle.
    ///
    /// The internal divider is incremented every T-Cycle. When the timer
    /// is enabled, TIMA is incremented when the selected divider bit
    /// transitions from `1` to `0` (falling edge).
    pub(crate) fn tick(&mut self) {
        let timer_enabled = self.tac & (1 << 2) != 0;

        let div_bit = self.get_frequency_bit();

        let prev_bit = (self.div >> div_bit) & 0x01;

        self.div += 1;

        let actual_bit = (self.div >> div_bit) & 0x01;

        if timer_enabled && prev_bit == 1 && actual_bit == 0 {
            self.increment_tima();
        }
    }
}
