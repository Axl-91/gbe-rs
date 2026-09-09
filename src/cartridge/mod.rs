pub mod mbc1;

use std::fs;
use std::io;
use std::path::Path;

use crate::memory::map::RAM_BANK_SIZE;
use crate::memory::map::{
    CARTRIDGE_RAM_END, CARTRIDGE_RAM_START, CARTRIDGE_ROM_END, CARTRIDGE_ROM_START,
};
use mbc1::Mbc1;

const DISABLED_RAM_VALUE: u8 = 0xFF;
const RAM_SIZE_CODE_ADDRESS: usize = 0x0149;

pub struct Cartridge {
    rom: Vec<u8>,
    ram: Vec<u8>,
    mbc: Mbc1,
}

impl Cartridge {
    pub fn new(rom: Vec<u8>, ram_size: usize) -> Self {
        Self {
            rom,
            ram: vec![0; ram_size],
            mbc: Mbc1::new(),
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let rom = fs::read(path)?;

        if rom.len() <= RAM_SIZE_CODE_ADDRESS {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "ROM is too small to contain the cartridge header",
            ));
        }

        // Header parsing
        let ram_size = match rom[RAM_SIZE_CODE_ADDRESS] {
            0x00 => 0,
            0x01 => RAM_BANK_SIZE / 4,
            0x02 => RAM_BANK_SIZE,
            0x03 => RAM_BANK_SIZE * 4,
            0x04 => RAM_BANK_SIZE * 16,
            0x05 => RAM_BANK_SIZE * 8,
            _ => 0,
        };

        Ok(Self::new(rom, ram_size))
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            CARTRIDGE_ROM_START..=CARTRIDGE_ROM_END => {
                let real_address = self.mbc.read(address);
                self.rom[real_address]
            }
            CARTRIDGE_RAM_START..=CARTRIDGE_RAM_END => {
                if self.mbc.is_ram_enabled() {
                    let bank_selected = self.mbc.get_ram_bank() as usize;
                    let offset = (address - CARTRIDGE_RAM_START) as usize;
                    let ram_address = (bank_selected * RAM_BANK_SIZE) + offset;

                    self.ram[ram_address]
                } else {
                    DISABLED_RAM_VALUE
                }
            }
            _ => unreachable!("Invalid cartridge address: {address:#06X}"),
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            CARTRIDGE_ROM_START..=CARTRIDGE_ROM_END => {
                self.mbc.write(address, value);
            }
            CARTRIDGE_RAM_START..=CARTRIDGE_RAM_END => {
                if self.mbc.is_ram_enabled() {
                    let bank_selected = self.mbc.get_ram_bank() as usize;
                    let offset = (address - CARTRIDGE_RAM_START) as usize;
                    let ram_address = (bank_selected * RAM_BANK_SIZE) + offset;

                    self.ram[ram_address] = value;
                }
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

    const RAM_DISABLE_VALUE: u8 = 0x00;

    fn create_test_cartridge(rom: Option<Vec<u8>>, ram_size: usize) -> Cartridge {
        match rom {
            Some(rom) => Cartridge::new(rom, ram_size),
            None => Cartridge::new(vec![0; ROM_BANK_SIZE * 4], ram_size),
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

            // Nothing was written on RAM
            let offset = address - CARTRIDGE_RAM_START;
            assert_eq!(cartridge.ram[offset as usize], 0x00);

            // Can't access that address on read
            assert_eq!(cartridge.read(address), DISABLED_RAM_VALUE);
        }

        #[test]
        fn read_ram_returns_disabled_value_when_disabled() {
            let mut rng = rand::rng();

            let address: u16 = rng.random_range(CARTRIDGE_RAM_START..=CARTRIDGE_RAM_END);

            let cartridge = create_test_cartridge(None, RAM_BANK_SIZE);

            assert_eq!(cartridge.read(address), DISABLED_RAM_VALUE);
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

            cartridge.write(enable_ram_address, RAM_DISABLE_VALUE);

            assert_eq!(cartridge.read(address), DISABLED_RAM_VALUE);

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
        fn creates_ram_with_size_from_header() {
            let ram_size_codes = [
                (0x00, 0),
                (0x01, RAM_BANK_SIZE / 4),
                (0x02, RAM_BANK_SIZE),
                (0x03, RAM_BANK_SIZE * 4),
                (0x04, RAM_BANK_SIZE * 16),
                (0x05, RAM_BANK_SIZE * 8),
            ];

            for (code, expected_size) in ram_size_codes {
                let mut rom = vec![0; ROM_BANK_SIZE * 2];
                rom[RAM_SIZE_CODE_ADDRESS] = code;

                let path = create_temp_rom(&rom);

                let cartridge = Cartridge::from_file(&path).unwrap();

                assert_eq!(cartridge.ram.len(), expected_size);

                fs::remove_file(path).unwrap();
            }
        }
    }
}
