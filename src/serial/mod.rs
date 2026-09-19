//! Emulates the Game Boy serial communication hardware.
//!
//! The serial interface is controlled through two memory-mapped registers:
//! - `FF01` (SB): Serial transfer data.
//! - `FF02` (SC): Serial transfer control.
//!
//! This implementation captures the byte written to the serial interface
//! when a transfer is started. The captured byte can then be retrieved
//! through [`Serial::take_output`].

pub struct Serial {
    data: u8,
    control: u8,
    output: Option<u8>,
}

impl Serial {
    /// Creates a new serial interface with all registers cleared
    /// and no pending output.
    pub fn new() -> Self {
        Self {
            data: 0,
            control: 0,
            output: None,
        }
    }

    /// Reads the serial transfer data register (SB).
    pub fn read_data(&self) -> u8 {
        self.data
    }

    /// Reads the serial transfer control register (SC).
    pub fn read_control(&self) -> u8 {
        self.control
    }

    /// Writes a value to the serial transfer data register (SB).
    pub fn write_data(&mut self, value: u8) {
        // panic!();
        self.data = value;
    }

    /// Writes a value to the serial transfer control register (SC).
    ///
    /// If bit 7 is set, a serial transfer is started and the current
    /// value of the data register is made available as output.
    pub fn write_control(&mut self, value: u8) {
        self.control = value;

        // Check if bit 7 is active.
        // This bit starts the serial transfer.
        if value & 0x80 != 0 {
            self.output = Some(self.data);
        }
    }

    /// Retrieves and clears the pending serial output.
    ///
    /// Returns `Some(byte)` when a new byte has been produced by a
    /// serial transfer, or `None` when there is no pending output.
    pub fn take_output(&mut self) -> Option<u8> {
        self.output.take()
    }
}

impl Default for Serial {
    fn default() -> Self {
        Self::new()
    }
}
