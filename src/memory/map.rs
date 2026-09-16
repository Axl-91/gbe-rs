//! Game Boy memory map.
//!
//! Defines address ranges and sizes for the different memory regions.

pub const CARTRIDGE_ROM_START: u16 = 0x0000;
pub const CARTRIDGE_ROM_END: u16 = 0x7FFF;

pub const VRAM_START: u16 = 0x8000;
pub const VRAM_END: u16 = 0x9FFF;

pub const CARTRIDGE_RAM_START: u16 = 0xA000;
pub const CARTRIDGE_RAM_END: u16 = 0xBFFF;

pub const WRAM_START: u16 = 0xC000;
pub const WRAM_END: u16 = 0xDFFF;

pub const RAM_BANK_SIZE: usize = (CARTRIDGE_RAM_END - CARTRIDGE_RAM_START + 1) as usize;
pub const ROM_BANK_SIZE: usize = 0x4000;

pub const INTERRUPT_FLAG_ADDRESS: u16 = 0xFF0F;
pub const INTERRUPT_ENABLE_ADDRESS: u16 = 0xFFFF;

pub const TIMER_DIV_ADDRESS: u16 = 0xFF04;
pub const TIMER_TIMA_ADDRESS: u16 = 0xFF05;
pub const TIMER_TMA_ADDRESS: u16 = 0xFF06;
pub const TIMER_TAC_ADDRESS: u16 = 0xFF07;
