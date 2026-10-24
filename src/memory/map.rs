//! Game Boy memory map.
//!
//! Defines address ranges and sizes for the different memory regions.

// SIZES
pub const RAM_BANK_SIZE: usize = 0x2000;
pub const ROM_BANK_SIZE: usize = 0x4000;

// Cartridge type ranges (address 0x0147), grouped by MBC family.
// See https://gbdev.io/pandocs/The_Cartridge_Header.html#0147--cartridge-type
pub const ROM_ONLY: u8 = 0x00;

pub const MBC1_SIMPLE_START: u8 = 0x01;
pub const MBC1_SIMPLE_END: u8 = 0x02;
pub const MBC1_BATTERY: u8 = 0x03;

pub const MBC2_SIMPLE: u8 = 0x05;
pub const MBC2_BATTERY: u8 = 0x06;

pub const MBC3_RTC_START: u8 = 0x0F;
pub const MBC3_RTC_END: u8 = 0x10;

pub const MBC3_SIMPLE_START: u8 = 0x11;
pub const MBC3_SIMPLE_END: u8 = 0x12;
pub const MBC3_BATTERY: u8 = 0x13;

pub const MBC5_START: u8 = 0x19;
pub const MBC5_END: u8 = 0x1D;
pub const MBC5_BATTERY: u8 = 0x1B | 0x1E;

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

pub const BOOT_ROM_END: u16 = 0x00FF;
pub const BOOT_ROM_DISABLE_ADDRESS: u16 = 0xFF50;

// I/O Ranges START ->

pub const JOYPAD_ADDRESS: u16 = 0xFF00;

pub const SERIAL_DATA_ADDRESS: u16 = 0xFF01;
pub const SERIAL_CONTROL_ADDRESS: u16 = 0xFF02;

pub const TIMER_ADDRESS_START: u16 = 0xFF04;
pub const TIMER_ADDRESS_END: u16 = 0xFF07;

pub const INTERRUPT_FLAG_ADDRESS: u16 = 0xFF0F;

pub const APU_REGISTERS_START: u16 = 0xFF10;
pub const APU_REGISTERS_END: u16 = 0xFF26;

pub const WAVE_RAM_START: u16 = 0xFF30;
pub const WAVE_RAM_END: u16 = 0xFF3F;

pub const DMA_ADDRESS: u16 = 0xFF46;

pub const PPU_REGISTERS_START: u16 = 0xFF40;
pub const PPU_REGISTERS_END: u16 = 0xFF4B;

// I/O Ranges END <-

pub const HRAM_START: u16 = 0xFF80;
pub const HRAM_END: u16 = 0xFFFE;

pub const INTERRUPT_ENABLE_ADDRESS: u16 = 0xFFFF;
