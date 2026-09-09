use std::fs;
use std::io;

use crate::mbc1::Mbc1;

pub struct Cartridge {
    rom: Vec<u8>,
    mbc: Mbc1,
}

impl Cartridge {
    pub fn new(rom: Vec<u8>) -> Self {
        Self {
            rom,
            mbc: Mbc1::new(),
        }
    }

    pub fn from_file(path: &str) -> io::Result<Self> {
        let rom = fs::read(path)?;

        Ok(Self {
            rom,
            mbc: Mbc1::new(),
        })
    }

    pub fn read(&self, address: u16) -> u8 {
        let real_address = self.mbc.read(address);
        self.rom[real_address]
    }

    pub fn write(&mut self, address: u16, value: u8) {
        self.mbc.write(address, value);
    }
}

#[cfg(test)]
mod tests {
    const ROM_START: u16 = 0x0000;
    const ROM_END: u16 = 0x7FFF;
    const ROM_SIZE: usize = (ROM_END - ROM_START + 1) as usize;

    use super::*;
    use rand::RngExt;

    #[test]
    fn read_rom() {
        let mut rng = rand::rng();

        let address: u16 = rng.random_range(ROM_START..=ROM_END);
        let value: u8 = rng.random();

        let mut rom = vec![0; ROM_SIZE];
        rom[address as usize] = value;

        let cartridge = Cartridge::new(rom);

        assert_eq!(cartridge.read(address), value);
    }
}
