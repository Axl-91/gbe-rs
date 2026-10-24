use std::{
    fs,
    io::{self, ErrorKind},
    path::Path,
};

pub const BOOT_ROM_SIZE: usize = 0x100;

pub struct BootRom {
    data: [u8; BOOT_ROM_SIZE],
    enabled: bool,
}

impl BootRom {
    pub fn new(data: [u8; BOOT_ROM_SIZE]) -> Self {
        Self {
            data,
            enabled: true,
        }
    }

    pub fn from_file(path: impl AsRef<Path>) -> io::Result<Self> {
        let data = fs::read(path)?;

        if data.len() != BOOT_ROM_SIZE {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("boot ROM must be {BOOT_ROM_SIZE} bytes, got {}", data.len()),
            ));
        }

        let mut boot_rom = [0; BOOT_ROM_SIZE];
        boot_rom.copy_from_slice(&data);

        Ok(Self::new(boot_rom))
    }

    pub fn read(&self, address: u16) -> u8 {
        self.data[address as usize]
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngExt;

    fn random_data() -> [u8; BOOT_ROM_SIZE] {
        let mut rng = rand::rng();
        rng.random()
    }

    #[test]
    fn boot_rom_is_enabled_by_default() {
        let data = random_data();
        let boot_rom = BootRom::new(data);

        assert!(boot_rom.is_enabled());
    }

    #[test]
    fn reads_data_at_address() {
        let mut rng = rand::rng();
        let data = random_data();
        let address = rng.random_range(0..BOOT_ROM_SIZE) as u16;
        let expected = data[address as usize];

        let boot_rom = BootRom::new(data);

        assert_eq!(boot_rom.read(address), expected);
    }

    #[test]
    fn disable_turns_off_boot_rom() {
        let data = random_data();
        let mut boot_rom = BootRom::new(data);

        boot_rom.disable();

        assert!(!boot_rom.is_enabled());
    }
}
