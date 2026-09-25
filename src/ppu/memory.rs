use crate::memory::map::{OAM_END, OAM_START, VRAM_END, VRAM_START};
use crate::ppu::{Ppu, PpuMode};

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

impl Ppu {
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
