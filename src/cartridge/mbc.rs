//! Common interface for the different memory bank controllers, and the enum
//! that wraps the concrete implementation used by each cartridge.

use crate::cartridge::{mbc1::Mbc1, mbc2::Mbc2, mbc3::Mbc3, mbc5::Mbc5};

/// Behavior shared by any memory bank controller.
pub trait Mbc {
    fn read(&self, address: u16) -> usize;
    fn write(&mut self, address: u16, value: u8);

    fn read_ram(&self, address: u16) -> u8;
    fn write_ram(&mut self, address: u16, value: u8);

    fn has_battery(&self) -> bool;

    fn is_ram_enabled(&self) -> bool;

    fn get_ram_bank(&self) -> u8 {
        0
    }

    fn get_save_data(&self) -> Option<Vec<u8>>;
    fn load_save_data(&mut self, data: &[u8]) -> bool;
}

/// Wraps a cartridge's concrete MBC. Each variant carries the actual
/// struct; `Cartridge` only ever knows this type, never `Mbc1` or `Mbc2`
/// directly.
pub enum CartridgeMbc {
    Mbc1(Mbc1),
    Mbc2(Box<Mbc2>),
    Mbc3(Box<Mbc3>),
    Mbc5(Box<Mbc5>),
}

impl Mbc for CartridgeMbc {
    fn read(&self, address: u16) -> usize {
        match self {
            CartridgeMbc::Mbc1(m) => m.read(address),
            CartridgeMbc::Mbc2(m) => m.read(address),
            CartridgeMbc::Mbc3(m) => m.read(address),
            CartridgeMbc::Mbc5(m) => m.read(address),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match self {
            CartridgeMbc::Mbc1(m) => m.write(address, value),
            CartridgeMbc::Mbc2(m) => m.write(address, value),
            CartridgeMbc::Mbc3(m) => m.write(address, value),
            CartridgeMbc::Mbc5(m) => m.write(address, value),
        }
    }

    fn read_ram(&self, address: u16) -> u8 {
        match self {
            CartridgeMbc::Mbc1(m) => m.read_ram(address),
            CartridgeMbc::Mbc2(m) => m.read_ram(address),
            CartridgeMbc::Mbc3(m) => m.read_ram(address),
            CartridgeMbc::Mbc5(m) => m.read_ram(address),
        }
    }

    fn write_ram(&mut self, address: u16, value: u8) {
        match self {
            CartridgeMbc::Mbc1(m) => m.write_ram(address, value),
            CartridgeMbc::Mbc2(m) => m.write_ram(address, value),
            CartridgeMbc::Mbc3(m) => m.write_ram(address, value),
            CartridgeMbc::Mbc5(m) => m.write_ram(address, value),
        }
    }

    fn has_battery(&self) -> bool {
        match self {
            CartridgeMbc::Mbc1(m) => m.has_battery(),
            CartridgeMbc::Mbc2(m) => m.has_battery(),
            CartridgeMbc::Mbc3(m) => m.has_battery(),
            CartridgeMbc::Mbc5(m) => m.has_battery(),
        }
    }

    fn is_ram_enabled(&self) -> bool {
        match self {
            CartridgeMbc::Mbc1(m) => m.is_ram_enabled(),
            CartridgeMbc::Mbc2(m) => m.is_ram_enabled(),
            CartridgeMbc::Mbc3(m) => m.is_ram_enabled(),
            CartridgeMbc::Mbc5(m) => m.is_ram_enabled(),
        }
    }

    fn get_ram_bank(&self) -> u8 {
        match self {
            CartridgeMbc::Mbc1(m) => m.get_ram_bank(),
            CartridgeMbc::Mbc2(m) => m.get_ram_bank(),
            CartridgeMbc::Mbc3(m) => m.get_ram_bank(),
            CartridgeMbc::Mbc5(m) => m.get_ram_bank(),
        }
    }

    fn get_save_data(&self) -> Option<Vec<u8>> {
        match self {
            Self::Mbc1(mbc) => mbc.get_save_data(),
            Self::Mbc2(mbc) => mbc.get_save_data(),
            Self::Mbc3(mbc) => mbc.get_save_data(),
            Self::Mbc5(mbc) => mbc.get_save_data(),
        }
    }

    fn load_save_data(&mut self, data: &[u8]) -> bool {
        match self {
            Self::Mbc1(mbc) => mbc.load_save_data(data),
            Self::Mbc2(mbc) => mbc.load_save_data(data),
            Self::Mbc3(mbc) => mbc.load_save_data(data),
            Self::Mbc5(mbc) => mbc.load_save_data(data),
        }
    }
}
