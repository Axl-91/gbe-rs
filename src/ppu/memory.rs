use crate::memory::map::{OAM_END, OAM_START, VRAM_END, VRAM_START};
use crate::ppu::{Ppu, PpuMode, SCANLINE_CYCLES, TOTAL_LINES, VISIBLE_LINES};

pub(super) const LCDC_ADDRESS: u16 = 0xFF40;
pub(super) const STAT_ADDRESS: u16 = 0xFF41;
pub(super) const SCY_ADDRESS: u16 = 0xFF42;
pub(super) const SCX_ADDRESS: u16 = 0xFF43;
pub(super) const LY_ADDRESS: u16 = 0xFF44;
pub(super) const LYC_ADDRESS: u16 = 0xFF45;
pub(super) const DMA_ADDRESS: u16 = 0xFF46;
pub(super) const BGP_ADDRESS: u16 = 0xFF47;
pub(super) const OBP0_ADDRESS: u16 = 0xFF48;
pub(super) const OBP1_ADDRESS: u16 = 0xFF49;
pub(super) const WY_ADDRESS: u16 = 0xFF4A;
pub(super) const WX_ADDRESS: u16 = 0xFF4B;

/// `mode_cycles` of OAM search from which the CPU can no longer read VRAM.
/// VRAM locks a few dots before STAT reports mode 3.
const VRAM_BLOCK_START: u16 = 76;

/// `mode_cycles` of OAM search from which CPU writes to OAM pass again.
const OAM_WRITE_UNBLOCK_START: u16 = 76;

impl Ppu {
    /// OAM is locked during OAM search and drawing, and also in the last M-cycle
    /// of HBlank before a visible line (the visible LY has already changed).
    /// On the first line after the LCD is switched on it stays accessible until
    /// drawing starts.
    fn can_read_oam(&self) -> bool {
        if !self.is_lcd_enabled() {
            return true;
        }
        match self.visible_mode() {
            PpuMode::VBlank => true,
            PpuMode::HBlank => !self.is_last_mcycle_before_visible_line(),
            _ => false,
        }
    }

    /// True in the last 4 dots of an HBlank followed by a visible line.
    /// Line 143 is excluded because VBlank follows and OAM stays accessible.
    fn is_last_mcycle_before_visible_line(&self) -> bool {
        matches!(self.mode, PpuMode::HBlank)
            && self.ly + 1 < VISIBLE_LINES
            && self.mode_cycles + 4 >= self.hblank_duration
    }

    /// VRAM is locked during drawing and from `VRAM_BLOCK_START` in OAM search.
    /// On the first line after the LCD is switched on it stays accessible until
    /// drawing starts.
    fn can_read_vram(&self) -> bool {
        if !self.is_lcd_enabled() {
            return true;
        }
        match self.visible_mode() {
            PpuMode::HBlank | PpuMode::VBlank => true,
            PpuMode::OamSearch => self.mode_cycles < VRAM_BLOCK_START,
            PpuMode::Drawing => false,
        }
    }

    // Read VRAM without restrictions, used for Pixel Fetcher
    pub(super) fn read_vram(&self, address: u16) -> u8 {
        let offset = (address - VRAM_START) as usize;
        self.vram[offset]
    }

    /// OAM writes are blocked during OAM search and drawing, except in the last
    /// dots of OAM search, where the write lock is released before drawing starts.
    /// Unlike reads, they are not blocked in the last M-cycle of HBlank.
    fn can_write_oam(&self) -> bool {
        if !self.is_lcd_enabled() {
            return true;
        }
        match self.visible_mode() {
            PpuMode::HBlank | PpuMode::VBlank => true,
            PpuMode::OamSearch => self.mode_cycles >= OAM_WRITE_UNBLOCK_START,
            PpuMode::Drawing => false,
        }
    }

    /// VRAM writes are only blocked during drawing, unlike reads, which are also
    /// blocked in the last dots of OAM search.
    fn can_write_vram(&self) -> bool {
        !self.is_lcd_enabled() || !matches!(self.visible_mode(), PpuMode::Drawing)
    }

    /// Returns the LY value as observed by the CPU.
    ///
    /// On DMG, the CPU sees LY already incremented during the last M-cycle
    /// (4 dots) of every line that advances to the next one: HBlank on lines
    /// 0..=143 and VBlank on lines 144..=152. The internal `ly` field still
    /// changes at the line boundary.
    fn read_ly(&self) -> u8 {
        if !self.is_lcd_enabled() {
            return self.ly;
        }
        if self.ly == TOTAL_LINES - 1 {
            return 0;
        }

        let line_length = match self.mode {
            PpuMode::HBlank => self.hblank_duration,
            PpuMode::VBlank => SCANLINE_CYCLES,
            _ => return self.ly,
        };

        let in_last_m_cycle = self.mode_cycles + 4 >= line_length;

        if in_last_m_cycle && self.ly < TOTAL_LINES - 1 {
            self.ly + 1
        } else {
            self.ly
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            VRAM_START..=VRAM_END => {
                if self.can_read_vram() {
                    let offset = (address - VRAM_START) as usize;
                    self.vram[offset]
                } else {
                    0xFF
                }
            }

            OAM_START..=OAM_END => {
                if self.can_read_oam() {
                    let offset = (address - OAM_START) as usize;
                    self.oam[offset]
                } else {
                    0xFF
                }
            }

            LCDC_ADDRESS => self.lcdc,
            STAT_ADDRESS => self.read_stat(),
            LY_ADDRESS => self.read_ly(),
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
                if self.can_write_vram() {
                    let offset = (address - VRAM_START) as usize;
                    self.vram[offset] = value
                }
            }

            OAM_START..=OAM_END => {
                if self.can_write_oam() {
                    let offset = (address - OAM_START) as usize;
                    self.oam[offset] = value
                }
            }

            LCDC_ADDRESS => {
                let was_off = !self.is_lcd_enabled();
                self.lcdc = value;

                if !self.is_lcd_enabled() {
                    self.turn_off();
                } else if was_off {
                    self.turn_on();
                }
            }

            // For stat we only take bits 3-6
            STAT_ADDRESS => {
                self.stat = value & 0b0111_1000;
            }

            // LY is READ only
            LY_ADDRESS => {}

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
