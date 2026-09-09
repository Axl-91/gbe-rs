use std::fs;
use std::io;

pub struct Cartridge {
    rom: Vec<u8>,
}

impl Cartridge {
    pub fn new(rom: Vec<u8>) -> Self {
        Self { rom }
    }

    pub fn from_file(path: &str) -> io::Result<Self> {
        let rom = fs::read(path)?;

        Ok(Self { rom })
    }

    pub fn read(&self, address: u16) -> u8 {
        self.rom[address as usize]
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
