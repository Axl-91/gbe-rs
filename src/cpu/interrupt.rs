//! Game Boy CPU interrupt definitions.
//!
//! Defines the interrupt types and their corresponding vectors and flags.

use crate::{
    cpu::Cpu,
    memory::map::{INTERRUPT_ENABLE_ADDRESS, INTERRUPT_FLAG_ADDRESS},
};

/// Represents a Game Boy hardware interrupt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Interruption {
    VBlank,
    LcdStat,
    Timer,
    Serial,
    Joypad,
}

impl Interruption {
    /// Returns the interrupt vector address.
    pub fn vector(&self) -> u16 {
        match self {
            Interruption::VBlank => 0x0040,
            Interruption::LcdStat => 0x0048,
            Interruption::Timer => 0x0050,
            Interruption::Serial => 0x0058,
            Interruption::Joypad => 0x0060,
        }
    }
    pub fn bit(&self) -> u8 {
        match self {
            Interruption::VBlank => 0,
            Interruption::LcdStat => 1,
            Interruption::Timer => 2,
            Interruption::Serial => 3,
            Interruption::Joypad => 4,
        }
    }
    pub fn t_cycles(&self) -> u8 {
        20
    }
}

fn interruption_bit(bit: u8) -> Interruption {
    match bit {
        0 => Interruption::VBlank,
        1 => Interruption::LcdStat,
        2 => Interruption::Timer,
        3 => Interruption::Serial,
        4 => Interruption::Joypad,
        _ => unreachable!("No interruption for this bit"),
    }
}

impl Cpu {
    /// Returns the highest-priority interrupt that can currently be serviced.
    pub(crate) fn check_interruption(&mut self) -> Option<Interruption> {
        let interrupt_enable = self.bus.read(INTERRUPT_ENABLE_ADDRESS);
        let interrupt_flag = self.bus.read(INTERRUPT_FLAG_ADDRESS);

        let mut offset: u8 = 0;

        while offset <= 4 {
            let is_enabled = (interrupt_enable >> offset) & 0x01;
            let is_requested = (interrupt_flag >> offset) & 0x01;

            if (is_enabled == 1) && (is_requested == 1) {
                return Some(interruption_bit(offset));
            }
            offset += 1;
        }
        None
    }

    /// Services a pending interrupt.
    ///
    /// Real hardware spends 5 M-Cycles here: 3 internal cycles (the CPU begins
    /// fetching the next opcode before deciding to dispatch an interrupt
    /// instead, then spends two more cycles on internal decision-making) plus
    /// the 2 M-Cycles of the PC push.
    ///
    /// The high-byte push can hit `IE` (`0xFFFF`) if `SP` lands there, so the
    /// interrupt actually serviced is re-decided right after that write, not
    /// before it. If the corrupted `IE` no longer has any interrupt pending
    /// against `IF`, dispatch is cancelled entirely: PC jumps to `0x0000` and
    /// `IF` is left untouched. Otherwise the (possibly different) interrupt is
    /// serviced as normal.
    pub(crate) fn handle_interruption(&mut self, interruption: Interruption) -> u8 {
        self.ime = false;

        self.tick_internal();
        self.tick_internal();
        self.tick_internal();

        let pc = self.registers.get_pc();
        let sp = self.registers.get_sp();

        // Push the high byte first. This is the write that can corrupt IE.
        let sp_high = sp.wrapping_sub(1);
        self.tick_write(sp_high, (pc >> 8) as u8);

        // Re-evaluate which interrupt (if any) is still pending now that IE
        // may have changed. This is the vector that actually gets serviced.
        let final_interruption = self.check_interruption();

        // Push the low byte. This can also hit IE, but it's too late to
        // affect the decision we just made above.
        let sp_low = sp.wrapping_sub(2);
        self.tick_write(sp_low, pc as u8);
        self.registers.set_sp(sp_low);

        match final_interruption {
            Some(final_interruption) => {
                // Clear only the bit for the interrupt we're actually servicing.
                let interrupt_flag = self.bus.read(INTERRUPT_FLAG_ADDRESS);
                let clear_mask = !(0x01 << final_interruption.bit());

                self.bus
                    .write(INTERRUPT_FLAG_ADDRESS, interrupt_flag & clear_mask);
                self.registers.set_pc(final_interruption.vector());
            }
            None => {
                // The IE corruption cancelled the dispatch entirely: IF is
                // left untouched, and PC jumps to the null vector instead.
                self.registers.set_pc(0x0000);
            }
        }

        interruption.t_cycles()
    }
}
