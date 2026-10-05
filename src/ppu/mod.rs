//! Emulation of the Game Boy's PPU (Picture Processing Unit).
//!
//! This module models the PPU's memory (VRAM/OAM), its memory-mapped I/O
//! registers (LCDC, STAT, SCY/SCX, LY/LYC, BGP/OBP0/OBP1, WY/WX), and the
//! per-scanline mode state machine (`OamSearch` -> `Drawing` -> `HBlank`,
//! repeating for each visible line, then `VBlank` for the remaining

use crate::{
    memory::map::VRAM_START,
    ppu::{
        bg_fetcher::{BgFetcher, FetcherRequest},
        bg_fifo::BgFifo,
        sprite_fetcher::{SpriteFetcher, SpriteFetcherRequest},
        sprite_fifo::{SpriteFifo, SpritePixel},
        sprites::{OamSearcher, Sprite},
    },
};

const VRAM_SIZE: usize = 0x2000;
const OAM_SIZE: usize = 0xA0;

/// Number of visible scanlines (0..VISIBLE_LINES-1 are drawn to the screen).
const VISIBLE_LINES: u8 = 144;

/// Total number of scanlines per frame, including the VBlank period.
const TOTAL_LINES: u8 = 154;

const SCREEN_WIDTH: u8 = 160;

const SCANLINE_CYCLES: u16 = 456;
const OAM_SEARCH_CYCLES: u16 = 80;

/// Line 0 after enabling the LCD starts 2 T-cycles late
/// so its HBlank is 2 cycles shorter.
const LCD_ON_LINE0_LATE_CYCLES: u16 = 2;

pub(super) const SPRITE_SIZE: i16 = 8;

mod bg_fetcher;
mod bg_fifo;
mod memory;
mod registers;
mod sprite_fetcher;
mod sprite_fifo;
mod sprites;

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
#[derive(Clone, Copy, Debug)]
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

    // PPU memory
    vram: [u8; VRAM_SIZE],
    oam: [u8; OAM_SIZE],

    // PPU mode state
    mode: PpuMode,
    mode_cycles: u16,
    // LCD state
    lcd_switched_on: bool,

    // STAT state
    ly_eq_lyc: bool,
    stat_line: bool,

    // Rendering pipeline
    bg_fetcher: BgFetcher,
    sprite_fetcher: SpriteFetcher,
    bg_fifo: BgFifo,
    sprite_fifo: SpriteFifo,
    oam_searcher: OamSearcher,

    // Rendering state
    drawing_x: u8,
    scx_discard: u8,
    hblank_duration: u16,

    // Window state
    window_active: bool,
    window_y_triggered: bool,
    window_line_counter: u8,

    // Sprites state
    sprites: Vec<Sprite>,
    sprite_fetch_index: usize,

    framebuffer: Vec<u8>,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            // PPU registers
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

            // PPU memory
            vram: [0; VRAM_SIZE],
            oam: [0; OAM_SIZE],

            // PPU mode state
            mode: PpuMode::OamSearch,
            mode_cycles: 0,
            // LCD state
            lcd_switched_on: false,

            // STAT state
            ly_eq_lyc: false,
            stat_line: false,

            // Rendering pipeline
            bg_fetcher: BgFetcher::new(),
            bg_fifo: BgFifo::new(),
            sprite_fifo: SpriteFifo::new(),
            oam_searcher: OamSearcher::new(),
            sprite_fetcher: SpriteFetcher::new(),

            // Rendering state
            drawing_x: 0,
            scx_discard: 0,
            hblank_duration: 0,

            // Window state
            window_active: false,
            window_y_triggered: false,
            window_line_counter: 0,

            // Sprites state
            sprites: Vec::new(),
            sprite_fetch_index: 0,

            framebuffer: vec![0; SCREEN_WIDTH as usize * VISIBLE_LINES as usize],
        }
    }

    pub(super) fn turn_off(&mut self) {
        self.lcd_switched_on = false;
        self.mode = PpuMode::HBlank;
        self.ly = 0;
        self.mode_cycles = 0;
        self.oam_searcher.reset();
        self.sprite_fetcher.reset();
        self.sprite_fetch_index = 0;
        self.window_active = false;
        self.window_line_counter = 0;
        self.sprites.clear();
    }

    pub(super) fn turn_on(&mut self) {
        self.mode = PpuMode::OamSearch;
        self.mode_cycles = 2;
        self.ly_eq_lyc = self.ly == self.lyc;
        self.lcd_switched_on = true;
    }

    fn calculate_hblank_duration(&mut self) {
        // Once we finish drawing we calculate the duration of Hblank
        // Hblank = TOTAL SCANLINE - OamSearch - Drawing
        let mut duration = SCANLINE_CYCLES - OAM_SEARCH_CYCLES - self.mode_cycles;

        if self.lcd_switched_on {
            duration -= LCD_ON_LINE0_LATE_CYCLES;
            self.lcd_switched_on = false;
        }
        self.hblank_duration = duration;
    }

    fn push_bg_pixels_into_fifo(&mut self, low: u8, high: u8) {
        if self.bg_fifo.is_empty() {
            self.bg_fifo.push_tile(low, high);
            self.bg_fetcher.complete_push();
        }
    }

    fn tick_bg_fetcher(&mut self) {
        if !self.window_active && self.can_start_window() {
            self.window_active = true;
            self.bg_fifo.clear();
            self.bg_fetcher.reset();
            self.scx_discard = 0;
        }

        self.add_fetcher_context();
        let request = self.bg_fetcher.tick();

        match request {
            Some(FetcherRequest::ReadVram(address)) => {
                let value = self.read_vram(address);
                self.bg_fetcher.receive(value);
            }
            Some(FetcherRequest::Push { low, high }) => self.push_bg_pixels_into_fifo(low, high),
            None => {}
        }
    }

    fn apply_palette(&self, pixel_color: u8, palette: u8) -> u8 {
        (palette >> (pixel_color * 2)) & 0b11
    }

    fn pixel_mixer(&self, background_pixel: u8, sprite_pixel: SpritePixel) -> u8 {
        let sprite_pixel_color = sprite_pixel.get_color();

        if sprite_pixel_color != 0 && (!sprite_pixel.is_behind_bg() || background_pixel == 0) {
            let palette = if sprite_pixel.uses_obp1() {
                self.obp1
            } else {
                self.obp0
            };
            return self.apply_palette(sprite_pixel_color, palette);
        }
        self.apply_palette(background_pixel, self.bgp)
    }

    fn get_screen_position(&self) -> usize {
        self.ly as usize * SCREEN_WIDTH as usize + self.drawing_x as usize
    }

    fn consume_pixel(&mut self) -> Option<u8> {
        let mut pixel = self.bg_fifo.pop()?;

        if !self.is_bg_window_enabled() {
            pixel = 0;
        }

        if self.scx_discard > 0 {
            self.scx_discard -= 1;
            return None;
        } else if self.drawing_x < SCREEN_WIDTH {
            let index = self.get_screen_position();

            if let Some(sprite_pixel) = self.sprite_fifo.pop() {
                pixel = self.pixel_mixer(pixel, sprite_pixel)
            }

            self.framebuffer[index] = pixel;

            self.drawing_x += 1;
        }

        Some(pixel)
    }

    fn push_sprite_pixels_into_fifo(&mut self, low: u8, high: u8) {
        let sprite = self.sprite_fetcher.get_sprite();

        let sprite_x = sprite.get_x();
        let sprite_attrs = sprite.get_attributes();

        let real_x_pos = sprite_x as i16 - SPRITE_SIZE;
        let relative_x = real_x_pos - self.drawing_x as i16;

        if self.sprite_fifo.should_push_sprite(relative_x) {
            self.sprite_fifo
                .push_tile(relative_x, sprite_attrs, low, high);
        }
    }

    fn tick_sprite_fetcher(&mut self) {
        match self.sprite_fetcher.tick() {
            Some(SpriteFetcherRequest::ReadVram(address)) => {
                let value = self.vram[(address - VRAM_START) as usize];
                self.sprite_fetcher.receive(value);
            }

            Some(SpriteFetcherRequest::Push { high, low }) => {
                self.push_sprite_pixels_into_fifo(low, high);
                self.sprite_fetcher.complete_push();
            }

            None => {}
        }
    }

    fn tick_drawing(&mut self) {
        if self.sprite_fetcher.is_active() {
            self.tick_sprite_fetcher();
            return;
        }

        self.tick_bg_fetcher();
        let consumed_pixel = self.consume_pixel();

        if consumed_pixel.is_some() {
            self.update_sprite_fetcher_context();
        }
    }

    /// Executes the logic associated with the current PPU mode.
    fn execute_mode(&mut self) {
        match self.mode {
            PpuMode::OamSearch => {
                if let Some(index) = self.oam_searcher.tick() {
                    self.add_sprite(index);
                }
            }
            PpuMode::Drawing => self.tick_drawing(),
            _ => {}
        }
    }

    /// Checks whether the current PPU mode has completed its work
    /// and can transition to the next mode.
    fn can_advance_mode(&self) -> bool {
        match self.mode {
            PpuMode::OamSearch => self.mode_cycles == OAM_SEARCH_CYCLES,
            PpuMode::Drawing => self.drawing_x == SCREEN_WIDTH,
            PpuMode::HBlank => self.mode_cycles == self.hblank_duration,
            PpuMode::VBlank => self.mode_cycles == SCANLINE_CYCLES,
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
    /// - `OamSearch` always transitions to `Drawing` while
    ///   clearing the FIFO, Fetcher and OamSearcher.
    /// - `Drawing` calculates the `HBlank` duration and trasition to it.
    pub fn advance_mode(&mut self) {
        match self.mode {
            PpuMode::HBlank => {
                if self.window_active {
                    self.window_line_counter += 1;
                    self.window_active = false;
                }
                self.ly += 1;

                if self.ly == self.wy {
                    self.window_y_triggered = true;
                }

                if self.ly == VISIBLE_LINES {
                    self.mode = PpuMode::VBlank
                } else {
                    self.mode = PpuMode::OamSearch
                }
                // Clear sprite data for the next scanline.
                self.sprite_fetch_index = 0;
                self.sprites.clear();
            }

            PpuMode::VBlank => {
                self.ly += 1;

                if self.ly == TOTAL_LINES {
                    self.ly = 0;
                    self.mode = PpuMode::OamSearch;

                    self.window_line_counter = 0;
                    self.window_y_triggered = self.ly == self.wy;

                    self.sprite_fetch_index = 0;
                    self.sprite_fetcher.reset();
                }
            }

            PpuMode::OamSearch => {
                self.mode = PpuMode::Drawing;
                self.scx_discard = self.scx % 8;
                self.drawing_x = 0;

                self.bg_fifo.clear();
                self.bg_fetcher.reset();
                self.oam_searcher.reset();
                self.sprite_fifo.reset();

                self.update_sprite_fetcher_context();
            }

            PpuMode::Drawing => {
                self.calculate_hblank_duration();
                self.mode = PpuMode::HBlank
            }
        }

        // Reset values after changes
        self.mode_cycles = 0;
        self.ly_eq_lyc = self.ly == self.lyc;
    }

    /// Advances the PPU by one T-cycle.
    ///
    /// Executes the logic associated with the current mode and advances to
    /// the next mode when the current mode has completed. The LYC comparison
    /// and STAT interrupt line are then updated, and entering VBlank generates
    /// a VBlank interrupt.
    ///
    /// Returns the interrupts generated during this T-cycle.
    pub fn tick(&mut self) -> PpuInterruptions {
        let mut ppu_interruptions = PpuInterruptions::default();

        if !self.is_lcd_enabled() {
            return ppu_interruptions;
        }
        self.mode_cycles += 1;

        self.execute_mode();

        if !self.can_advance_mode() {
            return ppu_interruptions;
        }

        self.advance_mode();

        if self.update_stat_line() {
            ppu_interruptions.stat = true;
        }
        if self.ly == VISIBLE_LINES {
            ppu_interruptions.vblank = true
        }
        ppu_interruptions
    }

    pub fn get_framebuffer(&self) -> &[u8] {
        &self.framebuffer
    }
}

impl Default for Ppu {
    fn default() -> Self {
        Self::new()
    }
}
