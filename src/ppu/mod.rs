//! Emulation of the Game Boy's PPU (Picture Processing Unit).
//!
//! This module models the PPU's memory (VRAM/OAM), its memory-mapped I/O
//! registers (LCDC, STAT, SCY/SCX, LY/LYC, BGP/OBP0/OBP1, WY/WX), and the
//! per-scanline mode state machine (`OamSearch` -> `Drawing` -> `HBlank`,
//! repeating for each visible line, then `VBlank` for the remaining

use crate::ppu::fetcher::{Fetcher, FetcherRequest};

const VRAM_SIZE: usize = 0x2000;
const OAM_SIZE: usize = 0xA0;

/// Number of visible scanlines (0..VISIBLE_LINES-1 are drawn to the screen).
const VISIBLE_LINES: u8 = 144;

/// Total number of scanlines per frame, including the VBlank period.
const TOTAL_LINES: u8 = 154;

mod fetcher;
mod memory;
mod registers;

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

    fetcher: Fetcher,
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

            fetcher: Fetcher::new(),
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

    fn advance_request(&mut self) {
        self.add_fetcher_context();

        if let Some(FetcherRequest::ReadVram(address)) = self.fetcher.tick() {
            let value = self.read_vram(address);
            self.fetcher.receive(value);
        }
    }

    /// Advances the PPU by one T-cycle.
    ///
    /// Increments the cycle counter for the current mode. Once it reaches
    /// [`Ppu::mode_duration`], the counter resets and the PPU advances to
    /// its next mode ([`Ppu::advance_mode`]). The LYC comparison and STAT
    /// interrupt line are then updated, and entering VBlank generates a
    /// VBlank interrupt.
    ///
    /// Returns the interrupts generated during this T-cycle.
    pub fn tick(&mut self) -> PpuInterruptions {
        let mut ppu_interruptions = PpuInterruptions::default();

        if !self.is_lcd_enabled() {
            return ppu_interruptions;
        }

        self.mode_cycles += 1;

        match self.mode {
            PpuMode::OamSearch => {}
            PpuMode::Drawing => self.advance_request(),
            PpuMode::HBlank => {}
            PpuMode::VBlank => {}
        }

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
}

impl Default for Ppu {
    fn default() -> Self {
        Self::new()
    }
}
