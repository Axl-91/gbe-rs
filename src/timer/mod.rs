//! Game Boy timer.
//!
//! The timer is driven by the internal divider (the *system counter*) and is
//! controlled through the TAC register. TIMA is incremented on the falling
//! edge of the selected divider bit while the timer is enabled.
//!
//! Timer registers:
//!
//! - `FF04`: DIV — upper 8 bits of the internal divider.
//! - `FF05`: TIMA — timer counter.
//! - `FF06`: TMA — timer modulo.
//! - `FF07`: TAC — timer control and frequency selection.
//!
//! The system counter here is a 16 bit value incremented once per T-Cycle,
//! so DIV (its upper 8 bits) ticks at 16384 Hz and the frequency bits
//! selected by TAC are 9, 3, 5 and 7.
//!
//! Note on hardware revisions: the enable bit of TAC is ANDed *before* the
//! falling edge detector, which is the DMG circuit. On CGB the AND is placed
//! after the detector, so disabling the timer while the selected bit is set
//! does not produce a tick there.

#[cfg(test)]
mod tests;

/// Number of T-Cycles in one M-Cycle.
const M_CYCLE: u8 = 4;

/// State of the delayed TIMA reload.
///
/// When TIMA overflows it stays at `0x00` for one M-Cycle (cycle *A*), and
/// only on the next M-Cycle (cycle *B*) is TMA copied into TIMA and the
/// timer interrupt requested.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Reload {
    /// No overflow in flight.
    Idle,
    /// Cycle *A*: TIMA reads as `0x00`, reload pending in `n + 1` T-Cycles.
    Pending(u8),
    /// Cycle *B*: TMA has just been copied into TIMA and IF has been set.
    Loading(u8),
}

pub struct Timer {
    div: u16,
    tima: u8,
    tma: u8,
    tac: u8,
    reload: Reload,
}

pub enum TimerEvent {
    Overflow,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            div: 0,
            tima: 0,
            tma: 0,
            tac: 0,
            reload: Reload::Idle,
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

    /// Reads TAC. The three upper bits are unused and read back as `1`.
    pub(crate) fn read_tac(&self) -> u8 {
        self.tac | 0xF8
    }

    /// Writes to DIV (any value resets the whole system counter).
    ///
    /// Because the counter bits feed the multiplexer directly, clearing the
    /// counter can produce a falling edge on the selected bit and therefore
    /// an early timer tick, which may in turn overflow TIMA.
    pub(crate) fn write_div(&mut self) -> Option<TimerEvent> {
        let previous_edge = self.selected_edge();
        self.div = 0;

        let event = self.advance_reload();

        if previous_edge && !self.selected_edge() {
            self.increment_tima();
        }

        event
    }

    /// Writes to TIMA.
    ///
    /// - During cycle *A* the write cancels the pending overflow entirely:
    ///   TMA is not copied and no interrupt is requested.
    /// - During cycle *B* the write is ignored, since the load signal stays
    ///   active for the whole cycle.
    pub(crate) fn write_tima(&mut self, value: u8) {
        match self.reload {
            Reload::Loading(_) => {}
            Reload::Pending(_) => {
                self.reload = Reload::Idle;
                self.tima = value;
            }
            Reload::Idle => self.tima = value,
        }
    }

    /// Writes to TMA.
    ///
    /// TIMA constantly copies its input while the load signal is active, so
    /// a write during cycle *B* lands in TIMA as well.
    pub(crate) fn write_tma(&mut self, value: u8) {
        self.tma = value;

        if matches!(self.reload, Reload::Loading(_)) {
            self.tima = value;
        }
    }

    /// Writes to TAC.
    ///
    /// The value fed to the falling edge detector is
    /// `enable AND system_counter[selected_bit]`. Changing either the enable
    /// bit or the frequency selection can therefore drop that value from `1`
    /// to `0` and produce a timer tick immediately, without any counter
    /// increment taking place:
    ///
    /// - Switching to a frequency whose bit is currently unset ticks.
    /// - Disabling the timer while the selected bit is set ticks (DMG only).
    pub(crate) fn write_tac(&mut self, value: u8) -> Option<TimerEvent> {
        let previous_edge = self.selected_edge();

        self.tac = value & 0b111;

        if previous_edge && !self.selected_edge() {
            self.increment_tima();
        }

        None
    }

    /// Advances the timer by one T-Cycle.
    ///
    /// The pending reload is resolved first, then the system counter is
    /// incremented and the falling edge of the selected bit is detected.
    pub(crate) fn tick(&mut self) -> Option<TimerEvent> {
        let event = self.advance_reload();

        let previous_edge = self.selected_edge();
        self.div = self.div.wrapping_add(1);

        if previous_edge && !self.selected_edge() {
            self.increment_tima();
        }

        event
    }

    /// Value seen by the falling edge detector: the selected counter bit
    /// gated by the timer enable bit.
    fn selected_edge(&self) -> bool {
        let timer_enabled = self.tac & (1 << 2) != 0;
        let bit = self.get_frequency_bit();

        timer_enabled && (self.div >> bit) & 0x01 == 1
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
        match self.tac & 0b11 {
            0b00 => 9,
            0b01 => 3,
            0b10 => 5,
            0b11 => 7,
            _ => unreachable!(),
        }
    }

    /// Increments TIMA. On overflow TIMA becomes `0x00` and the reload is
    /// scheduled one M-Cycle later; the interrupt is *not* requested here.
    fn increment_tima(&mut self) {
        match self.tima.checked_add(1) {
            Some(value) => self.tima = value,
            None => {
                self.tima = 0;
                self.reload = Reload::Pending(M_CYCLE - 1);
            }
        }
    }

    /// Drives the reload state machine forward by one T-Cycle, returning the
    /// overflow event on the cycle where TMA is copied into TIMA.
    fn advance_reload(&mut self) -> Option<TimerEvent> {
        match self.reload {
            Reload::Idle => None,
            Reload::Pending(0) => {
                self.tima = self.tma;
                self.reload = Reload::Loading(M_CYCLE - 1);
                Some(TimerEvent::Overflow)
            }
            Reload::Pending(n) => {
                self.reload = Reload::Pending(n - 1);
                None
            }
            Reload::Loading(0) => {
                self.reload = Reload::Idle;
                None
            }
            Reload::Loading(n) => {
                self.reload = Reload::Loading(n - 1);
                None
            }
        }
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}
