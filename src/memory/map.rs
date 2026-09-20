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

pub const JOYPAD_ADDRESS: u16 = 0xFF00;

pub const SERIAL_DATA_ADDRESS: u16 = 0xFF01;
pub const SERIAL_CONTROL_ADDRESS: u16 = 0xFF02;

pub const TIMER_ADDRESS_START: u16 = 0xFF04;
pub const TIMER_ADDRESS_END: u16 = 0xFF07;

pub const INTERRUPT_FLAG_ADDRESS: u16 = 0xFF0F;

pub const HRAM_START: u16 = 0xFF80;
pub const HRAM_END: u16 = 0xFFFE;

pub const INTERRUPT_ENABLE_ADDRESS: u16 = 0xFFFF;
