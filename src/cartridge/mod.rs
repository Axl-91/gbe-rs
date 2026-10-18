//! Game Boy cartridge implementation.
//!
//! Handles cartridge ROM and RAM access through the memory bank controller (MBC).

pub mod mbc;
pub mod mbc1;
pub mod mbc2;
pub mod mbc3;

use std::fs;
use std::io;
use std::path::Path;

use crate::cartridge::mbc::{CartridgeMbc, Mbc};
use crate::cartridge::mbc3::Mbc3;
use crate::memory::map::MBC2_END;
use crate::memory::map::MBC2_START;
use crate::memory::map::MBC3_END;
use crate::memory::map::MBC3_START;
use crate::memory::map::{
    CARTRIDGE_RAM_END, CARTRIDGE_RAM_START, CARTRIDGE_ROM_END, CARTRIDGE_ROM_START, MBC1_END,
    MBC1_START, RAM_BANK_SIZE, ROM_ONLY,
};
use mbc1::Mbc1;

const RAM_SIZE_CODE_ADDRESS: usize = 0x0149;
const CARTRIDGE_TYPE_ADDRESS: usize = 0x0147;

/// Represents a Game Boy cartridge with ROM, RAM, and a memory bank controller.
pub struct Cartridge {
    rom: Vec<u8>,
    mbc: CartridgeMbc,
}

impl Cartridge {
    /// Creates a new cartridge with the given ROM and RAM size.
    pub fn new(rom: Vec<u8>, mbc: CartridgeMbc) -> Self {
        Self { rom, mbc }
    }

    fn select_mbc(mbc_type: u8, ram_size: usize) -> io::Result<CartridgeMbc> {
        let mbc = match mbc_type {
            ROM_ONLY | MBC1_START..=MBC1_END => CartridgeMbc::Mbc1(Mbc1::new(ram_size)),
            MBC2_START..=MBC2_END => CartridgeMbc::Mbc2(Box::default()),
            MBC3_START..=MBC3_END => CartridgeMbc::Mbc3(Box::new(Mbc3::new(ram_size))),
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Unsupported cartridge type: {mbc_type:#04X}"),
                ));
            }
        };

        Ok(mbc)
    }

    fn select_ram_size(ram_type: u8) -> usize {
        match ram_type {
            0x00 => 0,
            0x01 => RAM_BANK_SIZE / 4,
            0x02 => RAM_BANK_SIZE,
            0x03 => RAM_BANK_SIZE * 4,
            0x04 => RAM_BANK_SIZE * 16,
            0x05 => RAM_BANK_SIZE * 8,
            _ => 0,
        }
    }

    /// Loads a cartridge ROM from the given file path
    /// and initializes its RAM from the cartridge header.
    pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let rom = fs::read(path)?;

        if rom.len() <= RAM_SIZE_CODE_ADDRESS {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "ROM is too small to contain the cartridge header",
            ));
        }

        let ram_size = Self::select_ram_size(rom[RAM_SIZE_CODE_ADDRESS]);
        let mbc = Self::select_mbc(rom[CARTRIDGE_TYPE_ADDRESS], ram_size)?;

        Ok(Self::new(rom, mbc))
    }

    /// Reads a byte from the cartridge ROM or RAM address space.
    pub fn read(&self, address: u16) -> u8 {
        match address {
            CARTRIDGE_ROM_START..=CARTRIDGE_ROM_END => {
                let real_address = self.mbc.read(address) % self.rom.len();
                self.rom[real_address]
            }
            CARTRIDGE_RAM_START..=CARTRIDGE_RAM_END => self.mbc.read_ram(address),
            _ => unreachable!("Invalid cartridge address: {address:#06X}"),
        }
    }

    /// Writes a byte to the cartridge address space or its memory bank controller.
    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            CARTRIDGE_ROM_START..=CARTRIDGE_ROM_END => {
                self.mbc.write(address, value);
            }
            CARTRIDGE_RAM_START..=CARTRIDGE_RAM_END => {
                self.mbc.write_ram(address, value);
            }
            _ => unreachable!("Invalid cartridge address: {address:#06X}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mbc1::{
        BANK_HIGH_END, BANK_HIGH_START, BANKING_MODE_END, BANKING_MODE_START, RAM_ENABLE_END,
        RAM_ENABLE_START, RAM_ENABLE_VALUE,
    };
    use crate::memory::map::ROM_BANK_SIZE;

    use super::*;
    use rand::RngExt;

    fn create_test_cartridge(rom: Option<Vec<u8>>, ram_size: usize) -> Cartridge {
        let mbc = CartridgeMbc::Mbc1(Mbc1::new(ram_size));
        match rom {
            Some(rom) => Cartridge::new(rom, mbc),
            None => Cartridge::new(vec![0; ROM_BANK_SIZE * 4], mbc),
        }
    }

    fn create_temp_rom(rom: &[u8]) -> std::path::PathBuf {
        let mut rng = rand::rng();
        let filename = format!("gbe_test_{:016x}.gb", rng.random::<u64>());
        let path = std::env::temp_dir().join(filename);

        fs::write(&path, rom).unwrap();

        path
    }

    mod rom {
        use super::*;

        #[test]
        fn read_rom() {
            let mut rng = rand::rng();

            let rom_size = (CARTRIDGE_ROM_END - CARTRIDGE_ROM_START + 1) as usize;
            let mut rom = vec![0; rom_size];

            let address: u16 = rng.random_range(CARTRIDGE_ROM_START..=CARTRIDGE_ROM_END);
            let value: u8 = rng.random();

            rom[address as usize] = value;

            let cartridge = create_test_cartridge(Some(rom), RAM_BANK_SIZE);

            assert_eq!(cartridge.read(address), value);
        }
    }
    mod ram {
        use super::*;

        #[test]
        fn read_ram_when_enabled() {
            let mut rng = rand::rng();

            let address: u16 = rng.random_range(CARTRIDGE_RAM_START..=CARTRIDGE_RAM_END);
            let value: u8 = rng.random();

            let mut cartridge = create_test_cartridge(None, RAM_BANK_SIZE);
            let enable_ram_address = rng.random_range(RAM_ENABLE_START..=RAM_ENABLE_END);

            cartridge.write(enable_ram_address, RAM_ENABLE_VALUE);
            cartridge.write(address, value);

            assert_eq!(cartridge.read(address), value);
        }

        #[test]
        fn write_ram_is_ignored_when_disabled() {
            let mut rng = rand::rng();

            let address: u16 = rng.random_range(CARTRIDGE_RAM_START..=CARTRIDGE_RAM_END);
            let value: u8 = rng.random();

            let mut cartridge = create_test_cartridge(None, RAM_BANK_SIZE);

            cartridge.write(address, value);

            assert_eq!(cartridge.read(address), 0xFF);
        }

        #[test]
        fn read_ram_returns_disabled_value_when_disabled() {
            let mut rng = rand::rng();

            let address: u16 = rng.random_range(CARTRIDGE_RAM_START..=CARTRIDGE_RAM_END);

            let cartridge = create_test_cartridge(None, RAM_BANK_SIZE);

            assert_eq!(cartridge.read(address), 0xFF);
        }

        #[test]
        fn ram_content_is_preserved_when_disabled() {
            let mut rng = rand::rng();

            let address: u16 = rng.random_range(CARTRIDGE_RAM_START..=CARTRIDGE_RAM_END);
            let value: u8 = rng.random();

            let mut cartridge = create_test_cartridge(None, RAM_BANK_SIZE);

            let enable_ram_address = rng.random_range(RAM_ENABLE_START..=RAM_ENABLE_END);

            cartridge.write(enable_ram_address, RAM_ENABLE_VALUE);
            cartridge.write(address, value);

            assert_eq!(cartridge.read(address), value);

            cartridge.write(enable_ram_address, 0x00);

            assert_eq!(cartridge.read(address), 0xFF);

            cartridge.write(enable_ram_address, RAM_ENABLE_VALUE);

            assert_eq!(cartridge.read(address), value);
        }

        #[test]
        fn ram_banks_have_independent_content() {
            let mut rng = rand::rng();

            let address: u16 = rng.random_range(CARTRIDGE_RAM_START..=CARTRIDGE_RAM_END);
            let value_bank_0: u8 = rng.random();
            let value_bank_1: u8 = rng.random();

            let ram_size = RAM_BANK_SIZE * 4;
            let mut cartridge = create_test_cartridge(None, ram_size);

            let enable_ram_address = rng.random_range(RAM_ENABLE_START..=RAM_ENABLE_END);
            let banking_mode_address = rng.random_range(BANKING_MODE_START..=BANKING_MODE_END);
            let bank_high_address = rng.random_range(BANK_HIGH_START..=BANK_HIGH_END);

            cartridge.write(enable_ram_address, RAM_ENABLE_VALUE);

            // Select RAM banking mode.
            cartridge.write(banking_mode_address, 0x01);

            // Select bank 0.
            cartridge.write(bank_high_address, 0);
            cartridge.write(address, value_bank_0);

            // Select bank 1.
            cartridge.write(bank_high_address, 1);
            cartridge.write(address, value_bank_1);

            // Bank 0 must keep its own value.
            cartridge.write(bank_high_address, 0);
            assert_eq!(cartridge.read(address), value_bank_0);

            // Bank 1 must keep its own value.
            cartridge.write(bank_high_address, 1);
            assert_eq!(cartridge.read(address), value_bank_1);
        }

        #[test]
        fn ram_uses_bank_zero_in_rom_banking_mode() {
            let mut rng = rand::rng();

            let address: u16 = rng.random_range(CARTRIDGE_RAM_START..=CARTRIDGE_RAM_END);
            let value: u8 = rng.random();
            let bank: u8 = rng.random_range(1..=0x03);

            let ram_size = RAM_BANK_SIZE * 4;
            let mut cartridge = create_test_cartridge(None, ram_size);

            let enable_ram_address = rng.random_range(RAM_ENABLE_START..=RAM_ENABLE_END);
            let bank_high_address = rng.random_range(BANK_HIGH_START..=BANK_HIGH_END);

            cartridge.write(enable_ram_address, RAM_ENABLE_VALUE);

            // ROM banking mode is the initial mode.
            cartridge.write(bank_high_address, bank);
            cartridge.write(address, value);

            // RAM must still use bank 0.
            cartridge.write(bank_high_address, 0);

            assert_eq!(cartridge.read(address), value);
        }
    }

    mod from_file {
        use super::*;

        #[test]
        fn loads_rom_from_file() {
            let mut rng = rand::rng();

            let address: u16 = rng.random_range(CARTRIDGE_ROM_START..=CARTRIDGE_ROM_END);
            let value: u8 = rng.random();

            let mut rom = vec![0; ROM_BANK_SIZE * 2];
            rom[address as usize] = value;

            let path = create_temp_rom(&rom);

            let cartridge = Cartridge::from_file(&path).unwrap();

            assert_eq!(cartridge.read(address), value);

            fs::remove_file(path).unwrap();
        }

        #[test]
        fn selects_ram_size_from_header() {
            let ram_size_codes = [
                (0x00, 0),
                (0x01, RAM_BANK_SIZE / 4),
                (0x02, RAM_BANK_SIZE),
                (0x03, RAM_BANK_SIZE * 4),
                (0x04, RAM_BANK_SIZE * 16),
                (0x05, RAM_BANK_SIZE * 8),
            ];

            for (code, expected_size) in ram_size_codes {
                assert_eq!(
                    Cartridge::select_ram_size(code),
                    expected_size,
                    "Unexpected RAM size for header code {code:#04X}"
                );
            }
        }
    }
}
