//! Game Boy memory map.
//!
//! Defines address ranges and sizes for the different memory regions.

// SIZES
pub const RAM_BANK_SIZE: usize = 0x2000;
pub const ROM_BANK_SIZE: usize = 0x4000;

// ADDRESSES
pub const CARTRIDGE_ROM_START: u16 = 0x0000;
pub const CARTRIDGE_ROM_END: u16 = 0x7FFF;

pub const VRAM_START: u16 = 0x8000;
pub const VRAM_END: u16 = 0x9FFF;

pub const CARTRIDGE_RAM_START: u16 = 0xA000;
pub const CARTRIDGE_RAM_END: u16 = 0xBFFF;

pub const WRAM_START: u16 = 0xC000;
pub const WRAM_END: u16 = 0xDFFF;

pub const ECHO_RAM_START: u16 = 0xE000;
pub const ECHO_RAM_END: u16 = 0xFDFF;

pub const OAM_START: u16 = 0xFE00;
pub const OAM_END: u16 = 0xFE9F;

pub const UNUSABLE_MEMORY_START: u16 = 0xFEA0;
pub const UNUSABLE_MEMORY_END: u16 = 0xFEFF;

// I/O Ranges START ->
pub const JOYPAD_ADDRESS: u16 = 0xFF00;

pub const SERIAL_DATA_ADDRESS: u16 = 0xFF01;
pub const SERIAL_CONTROL_ADDRESS: u16 = 0xFF02;

pub const TIMER_ADDRESS_START: u16 = 0xFF04;
pub const TIMER_ADDRESS_END: u16 = 0xFF07;

pub const INTERRUPT_FLAG_ADDRESS: u16 = 0xFF0F;
// I/O Ranges END <-

pub const HRAM_START: u16 = 0xFF80;
pub const HRAM_END: u16 = 0xFFFE;

pub const INTERRUPT_ENABLE_ADDRESS: u16 = 0xFFFF;
