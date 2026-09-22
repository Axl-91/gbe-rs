use crate::cartridge::Cartridge;
use crate::cartridge::{mbc::CartridgeMbc, mbc1::Mbc1};
use crate::memory::MemoryBus;
use crate::memory::map::{
    CARTRIDGE_ROM_END, CARTRIDGE_ROM_START, DMA_ADDRESS, OAM_START, RAM_BANK_SIZE, ROM_BANK_SIZE,
    VRAM_END, VRAM_START, WRAM_END, WRAM_START,
};
use rand::RngExt;

mod bus;
