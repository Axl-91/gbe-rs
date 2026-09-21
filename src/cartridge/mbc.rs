//! Common interface for the different memory bank controllers, and the enum
//! that wraps the concrete implementation used by each cartridge.

use crate::cartridge::mbc1::Mbc1;

/// Behavior shared by any memory bank controller.
pub trait Mbc {
    fn read(&self, address: u16) -> usize;
    fn write(&mut self, address: u16, value: u8);
    fn is_ram_enabled(&self) -> bool;
    fn get_ram_bank(&self) -> u8 {
        0
    }
}

/// Wraps a cartridge's concrete MBC. Each variant carries the actual
/// struct; `Cartridge` only ever knows this type, never `Mbc1` or `Mbc2`
/// directly.
pub enum CartridgeMbc {
    Mbc1(Mbc1),
    // Mbc2(Mbc2),
    // Mbc3(Mbc3),
}

impl Mbc for CartridgeMbc {
    fn read(&self, address: u16) -> usize {
        match self {
            CartridgeMbc::Mbc1(m) => m.read(address),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match self {
            CartridgeMbc::Mbc1(m) => m.write(address, value),
        }
    }

    fn is_ram_enabled(&self) -> bool {
        match self {
            CartridgeMbc::Mbc1(m) => m.is_ram_enabled(),
        }
    }

    fn get_ram_bank(&self) -> u8 {
        match self {
            CartridgeMbc::Mbc1(m) => m.get_ram_bank(),
        }
    }
}
