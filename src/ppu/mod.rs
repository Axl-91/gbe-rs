//! Emulation of the Game Boy's PPU (Picture Processing Unit).
//!
//! This module models the PPU's memory (VRAM/OAM), its memory-mapped I/O
//! registers (LCDC, STAT, SCY/SCX, LY/LYC, BGP/OBP0/OBP1, WY/WX), and the
//! per-scanline mode state machine (`OamSearch` -> `Drawing` -> `HBlank`,
//! repeating for each visible line, then `VBlank` for the remaining

// TODO: Once all functions are used we can delete this
#![allow(dead_code)]

use crate::memory::map::{OAM_END, OAM_START, VRAM_END, VRAM_START};

const VRAM_SIZE: usize = 0x2000;
const OAM_SIZE: usize = 0xA0;

const LCDC_ADDRESS: u16 = 0xFF40;
const STAT_ADDRESS: u16 = 0xFF41;
const SCY_ADDRESS: u16 = 0xFF42;
const SCX_ADDRESS: u16 = 0xFF43;
const LY_ADDRESS: u16 = 0xFF44;
const LYC_ADDRESS: u16 = 0xFF45;
const DMA_ADDRESS: u16 = 0xFF46;
const BGP_ADDRESS: u16 = 0xFF47;
const OBP0_ADDRESS: u16 = 0xFF48;
const OBP1_ADDRESS: u16 = 0xFF49;
const WY_ADDRESS: u16 = 0xFF4A;
const WX_ADDRESS: u16 = 0xFF4B;

const LCDC_ENABLE: u8 = 7;
const WINDOW_TILE_MAP: u8 = 6;
const WINDOW_ENABLE: u8 = 5;
const BG_WINDOW_TILE_DATA: u8 = 4;
const BG_TILE_MAP: u8 = 3;
const OBJ_SIZE: u8 = 2;
const OBJ_ENABLE: u8 = 1;
const BG_WINDOW_ENABLE: u8 = 0;

/// Number of visible scanlines (0..VISIBLE_LINES-1 are drawn to the screen).
const VISIBLE_LINES: u8 = 144;

/// Total number of scanlines per frame, including the VBlank period.
const TOTAL_LINES: u8 = 154;

#[cfg(test)]
mod tests;

#[derive(Default)]
pub struct PpuInterruptions {
    pub vblank: bool,
    pub stat: bool,
}

/// The four PPU rendering modes, as reported in the STAT register.
///
/// Each frame cycles through, per line: `OamSearch` -> `Drawing` ->
/// `HBlank`, repeated for every visible line, followed by a single
/// `VBlank` mode that lasts for the remaining (non-visible) lines.
#[derive(Clone, Copy)]
enum PpuMode {
    HBlank = 0,
    VBlank = 1,
    OamSearch = 2,
    Drawing = 3,
}

/// Emulated Game Boy PPU: registers, VRAM/OAM, and mode timing.
pub struct Ppu {
    // PPU registers
    lcdc: u8,
    stat: u8,
    scy: u8,
    scx: u8,
    ly: u8,
    lyc: u8,
    bgp: u8,
    obp0: u8,
    obp1: u8,
    wy: u8,
    wx: u8,

    vram: [u8; VRAM_SIZE],
    oam: [u8; OAM_SIZE],

    mode: PpuMode,
    mode_cycles: u16,

    ly_eq_lyc: bool,
    stat_line: bool,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            lcdc: 0x91,
            stat: 0x85,
            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            bgp: 0xFC,
            obp0: 0,
            obp1: 0,
            wy: 0,
            wx: 0,

            vram: [0; VRAM_SIZE],
            oam: [0; OAM_SIZE],

            mode: PpuMode::OamSearch,
            mode_cycles: 0x00,

            ly_eq_lyc: false,
            stat_line: false,
        }
    }

    fn mode_duration(&self) -> u16 {
        match self.mode {
            PpuMode::HBlank => 204,
            PpuMode::VBlank => 456,
            PpuMode::OamSearch => 80,
            // TODO: Mode 3 duration is variable. 172 T-cycles is the minimum duration
            PpuMode::Drawing => 172,
        }
    }

    /// Advances the PPU to its next mode, updating `ly` as needed.
    ///
    /// - After `HBlank`, `ly` is incremented; if it reaches
    ///   [`VISIBLE_LINES`], the PPU enters `VBlank`, otherwise it
    ///   restarts the line with `OamSearch`.
    /// - After `VBlank`, `ly` is incremented; once it reaches
    ///   [`TOTAL_LINES`], `ly` wraps back to 0 and a new frame begins
    ///   with `OamSearch`.
    /// - `OamSearch` always transitions to `Drawing`.
    /// - `Drawing` always transitions to `HBlank`.
    pub fn advance_mode(&mut self) {
        match self.mode {
            PpuMode::HBlank => {
                self.ly += 1;

                if self.ly == VISIBLE_LINES {
                    self.mode = PpuMode::VBlank
                } else {
                    self.mode = PpuMode::OamSearch
                }
            }

            PpuMode::VBlank => {
                self.ly += 1;

                if self.ly == TOTAL_LINES {
                    self.ly = 0;
                    self.mode = PpuMode::OamSearch;
                }
            }

            PpuMode::OamSearch => self.mode = PpuMode::Drawing,

            PpuMode::Drawing => {
                self.mode = PpuMode::HBlank;
            }
        }
    }

    /// Recalculates the combined STAT signal (`stat_line`) and returns
    /// `true` only on its rising edge (no condition met -> at least one
    /// met), which is when a STAT interrupt should be requested.
    ///
    /// Implements "STAT IRQ Blocking": while `stat_line` stays `true`,
    /// no new edge is detected, even if other conditions activate meanwhile.
    /// It can only rise again after falling back to `false` first.
    fn update_stat_line(&mut self) -> bool {
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

    // Computes the current state of the combined STAT signal, without
    /// mutating any state.
    ///
    /// Returns `true` if the LCD is enabled and at least one enabled STAT
    /// condition currently holds
    fn compute_stat_line(&self) -> bool {
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

    /// Advances the PPU by one T-cycle.
    ///
    /// Increments the cycle counter for the current mode; once it
    /// reaches [`Ppu::mode_duration`], the counter resets, the PPU
    /// advances to its next mode ([`Ppu::advance_mode`]), and the LYC
    /// match flag is refreshed ([`Ppu::update_lyc_match`]).
    ///
    /// Returns `true` if a STAT interrupt should be requested as a
    /// result of this cycle's mode/LYC transition, `false` otherwise
    /// (including when no mode transition occurred this cycle).
    pub fn tick(&mut self) -> PpuInterruptions {
        let mut ppu_interruptions = PpuInterruptions::default();

        if !self.is_lcd_enabled() {
            return ppu_interruptions;
        }

        self.mode_cycles += 1;

        if self.mode_cycles < self.mode_duration() {
            return ppu_interruptions;
        }

        self.mode_cycles = 0;

        self.advance_mode();
        self.ly_eq_lyc = self.ly == self.lyc;

        if self.update_stat_line() {
            ppu_interruptions.stat = true;
        }
        if self.ly == VISIBLE_LINES {
            ppu_interruptions.vblank = true
        }
        ppu_interruptions
    }

    fn is_lcd_enabled(&self) -> bool {
        self.lcdc & (1 << LCDC_ENABLE) != 0
    }

    fn is_window_tile_map(&self) -> bool {
        self.lcdc & (1 << WINDOW_TILE_MAP) != 0
    }

    fn is_window_enabled(&self) -> bool {
        self.lcdc & (1 << WINDOW_ENABLE) != 0
    }

    fn is_bg_window_tile_data(&self) -> bool {
        self.lcdc & (1 << BG_WINDOW_TILE_DATA) != 0
    }

    fn is_bg_tile_map(&self) -> bool {
        self.lcdc & (1 << BG_TILE_MAP) != 0
    }

    fn is_obj_size(&self) -> bool {
        self.lcdc & (1 << OBJ_SIZE) != 0
    }

    fn is_obj_enabled(&self) -> bool {
        self.lcdc & (1 << OBJ_ENABLE) != 0
    }

    fn is_bg_window_enabled(&self) -> bool {
        self.lcdc & (1 << BG_WINDOW_ENABLE) != 0
    }

    fn read_stat(&self) -> u8 {
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

    fn is_oam_accessible(&self) -> bool {
        !self.is_lcd_enabled() || matches!(self.mode, PpuMode::HBlank | PpuMode::VBlank)
    }

    fn is_vram_accessible(&self) -> bool {
        !self.is_lcd_enabled()
            || matches!(
                self.mode,
                PpuMode::HBlank | PpuMode::VBlank | PpuMode::OamSearch
            )
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            VRAM_START..=VRAM_END => {
                if self.is_vram_accessible() {
                    let offset = (address - VRAM_START) as usize;
                    self.vram[offset]
                } else {
                    0xFF
                }
            }

            OAM_START..=OAM_END => {
                if self.is_oam_accessible() {
                    let offset = (address - OAM_START) as usize;
                    self.oam[offset]
                } else {
                    0xFF
                }
            }

            LCDC_ADDRESS => self.lcdc,
            STAT_ADDRESS => self.read_stat(),
            LY_ADDRESS => self.ly,

            SCY_ADDRESS => self.scy,
            SCX_ADDRESS => self.scx,
            LYC_ADDRESS => self.lyc,
            BGP_ADDRESS => self.bgp,
            OBP0_ADDRESS => self.obp0,
            OBP1_ADDRESS => self.obp1,
            WY_ADDRESS => self.wy,
            WX_ADDRESS => self.wx,

            // DMA is handled separately.
            DMA_ADDRESS => 0,

            _ => unreachable!("Invalid PPU address: {address:#06X}"),
        }
    }

    // This function is for OAM DMA transfer which should not have any restriction
    pub fn dma_write_oam(&mut self, address: u16, value: u8) {
        let offset = (address - OAM_START) as usize;
        self.oam[offset] = value
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            VRAM_START..=VRAM_END => {
                if self.is_vram_accessible() {
                    let offset = (address - VRAM_START) as usize;
                    self.vram[offset] = value
                }
            }

            OAM_START..=OAM_END => {
                if self.is_oam_accessible() {
                    let offset = (address - OAM_START) as usize;
                    self.oam[offset] = value
                }
            }

            LCDC_ADDRESS => {
                let was_off = !self.is_lcd_enabled();
                self.lcdc = value;
                if !self.is_lcd_enabled() {
                    self.mode = PpuMode::HBlank;
                    self.ly = 0;
                    self.mode_cycles = 0;
                }
                if was_off && self.is_lcd_enabled() {
                    self.ly_eq_lyc = self.ly == self.lyc;
                }
            }

            // For stat we only take bits 3-6
            STAT_ADDRESS => {
                self.stat = value & 0b0111_1000;
            }

            LY_ADDRESS => self.ly = 0x00,

            SCY_ADDRESS => self.scy = value,
            SCX_ADDRESS => self.scx = value,
            LYC_ADDRESS => {
                self.lyc = value;
                if self.is_lcd_enabled() {
                    self.ly_eq_lyc = self.ly == self.lyc;
                }
            }
            BGP_ADDRESS => self.bgp = value,
            OBP0_ADDRESS => self.obp0 = value,
            OBP1_ADDRESS => self.obp1 = value,
            WY_ADDRESS => self.wy = value,

            WX_ADDRESS => self.wx = value,

            // DMA is handled separately.
            DMA_ADDRESS => {}

            _ => unreachable!("Invalid PPU address: {address:#06X}"),
        }
    }
}

impl Default for Ppu {
    fn default() -> Self {
        Self::new()
    }
}
