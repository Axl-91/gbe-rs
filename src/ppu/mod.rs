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

pub struct Ppu {
    vram: [u8; VRAM_SIZE],
    oam: [u8; OAM_SIZE],

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
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            vram: [0; VRAM_SIZE],
            oam: [0; OAM_SIZE],

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
        }
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
            STAT_ADDRESS => self.stat,
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
            STAT_ADDRESS => self.stat = value,
            LY_ADDRESS => self.ly = value,

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
