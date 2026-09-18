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

    pub(crate) fn handle_interruption(&mut self, interruption: Interruption) -> u8 {
        self.ime = false;

        // Real hardware always begins fetching the next opcode before
        // deciding to dispatch an interrupt instead; that discarded fetch,
        // plus two further internal decision cycles, account for 3
        // M-Cycles here. None of them touch the bus.
        self.tick_internal();
        self.tick_internal();
        self.tick_internal();

        let pc = self.registers.get_pc();

        // Acknowledging (clearing) the IF bit is an internal CPU operation
        // that happens alongside the low-byte push below, not a bus access
        // of its own — so it must not consume an extra M-Cycle by itself.
        let interrupt_flag = self.bus.read(INTERRUPT_FLAG_ADDRESS);
        let clear_mask = !(0x01 << interruption.bit());
        self.bus
            .write(INTERRUPT_FLAG_ADDRESS, interrupt_flag & clear_mask);

        // Push the current PC onto the stack (2 M-Cycles: high byte, low byte).
        self.push_into_sp(pc);

        // Jump to the interrupt vector.
        self.registers.set_pc(interruption.vector());

        interruption.t_cycles()
    }
}
