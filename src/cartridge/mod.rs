pub mod mbc1;

use std::fs;
use std::io;

use crate::memory::map::{
    CARTRIDGE_RAM_END, CARTRIDGE_RAM_START, CARTRIDGE_ROM_END, CARTRIDGE_ROM_START,
};
use mbc1::Mbc1;

const RAM_SIZE: u16 = 0x2000;
const DISABLED_RAM_VALUE: u8 = 0xFF;

pub struct Cartridge {
    rom: Vec<u8>,
    ram: Vec<u8>,
    mbc: Mbc1,
}

impl Cartridge {
    pub fn new(rom: Vec<u8>) -> Self {
        Self {
            rom,
            ram: vec![0; RAM_SIZE as usize],
            mbc: Mbc1::new(),
        }
    }

    pub fn from_file(path: &str) -> io::Result<Self> {
        let rom = fs::read(path)?;

        Ok(Self::new(rom))
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            CARTRIDGE_ROM_START..=CARTRIDGE_ROM_END => {
                let real_address = self.mbc.read(address);
                self.rom[real_address]
            }
            CARTRIDGE_RAM_START..=CARTRIDGE_RAM_END => {
                if self.mbc.is_ram_enabled() {
                    let offset = address - CARTRIDGE_RAM_START;
                    self.ram[offset as usize]
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
                    let offset = address - CARTRIDGE_RAM_START;
                    self.ram[offset as usize] = value;
                }
            }
            _ => unreachable!("Invalid cartridge address: {address:#06X}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngExt;

    const RAM_ENABLE_VALUE: u8 = 0x0A;
    const RAM_DISABLE_VALUE: u8 = 0x00;

    const RAM_ENABLE_START: u16 = 0x0000;
    const RAM_ENABLE_END: u16 = 0x1FFF;

    fn create_test_cartridge(rom: Option<Vec<u8>>) -> Cartridge {
        match rom {
            Some(rom) => Cartridge::new(rom),
            None => {
                let rom_size = (CARTRIDGE_ROM_END - CARTRIDGE_ROM_START + 1) as usize;

                Cartridge::new(vec![0; rom_size])
            }
        }
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

            let cartridge = create_test_cartridge(Some(rom));

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

            let mut cartridge = create_test_cartridge(None);
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

            let mut cartridge = create_test_cartridge(None);

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

            let cartridge = create_test_cartridge(None);

            assert_eq!(cartridge.read(address), DISABLED_RAM_VALUE);
        }

        #[test]
        fn ram_content_is_preserved_when_disabled() {
            let mut rng = rand::rng();

            let address: u16 = rng.random_range(CARTRIDGE_RAM_START..=CARTRIDGE_RAM_END);
            let value: u8 = rng.random();

            let mut cartridge = create_test_cartridge(None);

            cartridge.write(RAM_ENABLE_START, RAM_ENABLE_VALUE);
            cartridge.write(address, value);

            assert_eq!(cartridge.read(address), value);

            cartridge.write(RAM_ENABLE_START, RAM_DISABLE_VALUE);

            assert_eq!(cartridge.read(address), DISABLED_RAM_VALUE);

            cartridge.write(RAM_ENABLE_START, RAM_ENABLE_VALUE);

            assert_eq!(cartridge.read(address), value);
        }
    }
}
