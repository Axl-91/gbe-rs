// TODO: Once all functions are used we can delete this
#![allow(dead_code)]

use crate::ppu::{Ppu, PpuMode};

const LCDC_ENABLE: u8 = 7;
const WINDOW_TILE_MAP: u8 = 6;
const WINDOW_ENABLE: u8 = 5;
const BG_WINDOW_TILE_DATA: u8 = 4;
const BG_TILE_MAP: u8 = 3;
const OBJ_SIZE: u8 = 2;
const OBJ_ENABLE: u8 = 1;
const BG_WINDOW_ENABLE: u8 = 0;

impl Ppu {
    /// Computes the current state of the combined STAT signal, without
    /// mutating any state.
    ///
    /// Returns `true` if the LCD is enabled and at least one enabled STAT
    /// condition currently holds
    pub(super) fn compute_stat_line(&self) -> bool {
        if !self.is_lcd_enabled() {
            return false;
        }

        let mode_condition = match self.mode {
            PpuMode::HBlank => self.stat & (1 << 3) != 0,
            PpuMode::VBlank => self.stat & (1 << 4) != 0,
            PpuMode::OamSearch => self.stat & (1 << 5) != 0,
            PpuMode::Drawing => false,
        };

        let lyc_condition = self.ly_eq_lyc && self.stat & (1 << 6) != 0;

        mode_condition || lyc_condition
    }

    /// Recalculates the combined STAT signal (`stat_line`) and returns
    /// `true` only on its rising edge (no condition met -> at least one
    /// met), which is when a STAT interrupt should be requested.
    ///
    /// Implements "STAT IRQ Blocking": while `stat_line` stays `true`,
    /// no new edge is detected, even if other conditions activate meanwhile.
    /// It can only rise again after falling back to `false` first.
    pub(super) fn update_stat_line(&mut self) -> bool {
        let new_line = self.compute_stat_line();
        let rising_edge = new_line && !self.stat_line;
        self.stat_line = new_line;
        rising_edge
    }

    /// Recalculates the STAT signal after a register write and returns
    /// whether that write triggered a new STAT interrupt request.
    ///
    /// Must be called once right after any write that could affect
    /// `stat_line` (STAT, LYC, LCDC). Calling it again without an
    /// intervening state change will return `false`, since the edge
    /// was already consumed.
    pub fn has_interruptions(&mut self) -> bool {
        self.is_lcd_enabled() && self.update_stat_line()
    }

    pub(super) fn is_lcd_enabled(&self) -> bool {
        self.lcdc & (1 << LCDC_ENABLE) != 0
    }

    pub(super) fn is_window_tile_map(&self) -> bool {
        self.lcdc & (1 << WINDOW_TILE_MAP) != 0
    }

    pub(super) fn is_window_enabled(&self) -> bool {
        self.lcdc & (1 << WINDOW_ENABLE) != 0
    }

    pub(super) fn is_bg_window_tile_data(&self) -> bool {
        self.lcdc & (1 << BG_WINDOW_TILE_DATA) != 0
    }

    pub(super) fn is_bg_tile_map(&self) -> bool {
        self.lcdc & (1 << BG_TILE_MAP) != 0
    }

    pub(super) fn is_obj_size(&self) -> bool {
        self.lcdc & (1 << OBJ_SIZE) != 0
    }

    pub(super) fn is_obj_enabled(&self) -> bool {
        self.lcdc & (1 << OBJ_ENABLE) != 0
    }

    pub(super) fn is_bg_window_enabled(&self) -> bool {
        self.lcdc & (1 << BG_WINDOW_ENABLE) != 0
    }

    pub(super) fn read_stat(&self) -> u8 {
        let mut stat = self.stat;

        // bit 2 -> ly==lyc
        if self.ly_eq_lyc {
            stat |= 0x04;
        }
        // bit 0-1 -> PPU Mode
        stat |= self.mode as u8;

        // Bit 7 is always 1
        stat | 0b1000_0000
    }
}
