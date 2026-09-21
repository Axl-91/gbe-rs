// TODO: Once all functions are used we can delete this
#![allow(dead_code)]

use crate::memory::map::*;

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

const VISIBLE_LINES: u8 = 144;
const TOTAL_LINES: u8 = 154;

#[cfg(test)]
mod tests;

#[derive(Clone, Copy)]
enum PpuMode {
    HBlank = 0,
    VBlank = 1,
    OamSearch = 2,
    Drawing = 3,
}

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

    pub fn tick(&mut self) {
        self.mode_cycles += 1;

        if self.mode_cycles < self.mode_duration() {
            return;
        }
        self.mode_cycles = 0;

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
        if self.ly == self.lyc {
            stat |= 0x04;
        }

        // bit 0-1 -> PPU Mode
        stat |= self.mode as u8;

        stat | 0x80
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            VRAM_START..=VRAM_END => {
                let offset = (address - VRAM_START) as usize;
                self.vram[offset]
            }

            OAM_START..=OAM_END => {
                let offset = (address - OAM_START) as usize;
                self.oam[offset]
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

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            VRAM_START..=VRAM_END => {
                let offset = (address - VRAM_START) as usize;
                self.vram[offset] = value
            }

            OAM_START..=OAM_END => {
                let offset = (address - OAM_START) as usize;
                self.oam[offset] = value
            }

            LCDC_ADDRESS => self.lcdc = value,

            // For stat we only take bits 3-6
            STAT_ADDRESS => self.stat = value & 0b0111_1000,

            LY_ADDRESS => self.ly = 0x00,

            SCY_ADDRESS => self.scy = value,
            SCX_ADDRESS => self.scx = value,
            LYC_ADDRESS => self.lyc = value,
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
