// const CARTRIDGE_ROM_START: u16 = 0x0000;
// const CARTRIDGE_ROM_END: u16 = 0x7FFF;

const VRAM_START: u16 = 0x8000;
const VRAM_END: u16 = 0x9FFF;

// const CARTRIDGE_RAM_START: u16 = 0xA000;
// const CARTRIDGE_RAM_END: u16 = 0xBFFF;

const WRAM_START: u16 = 0xC000;
const WRAM_END: u16 = 0xDFFF;

pub struct MemoryBus {
    vram: [u8; 0x2000],
    wram: [u8; 0x2000],
}

impl MemoryBus {
    pub fn new() -> Self {
        Self {
            vram: [0; 0x2000],
            wram: [0; 0x2000],
        }
    }
    pub fn read(&self, address: u16) -> u8 {
        match address {
            VRAM_START..=VRAM_END => {
                let offset = (address - VRAM_START) as usize;
                self.vram[offset]
            }

            WRAM_START..=WRAM_END => {
                let offset = (address - WRAM_START) as usize;
                self.wram[offset]
            }

            _ => 0,
        }
    }
    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            VRAM_START..=VRAM_END => {
                self.vram[(address - VRAM_START) as usize] = value;
            }

            WRAM_START..=WRAM_END => {
                self.wram[(address - WRAM_START) as usize] = value;
            }

            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngExt;

    #[test]
    fn read_vram() {
        let bus = MemoryBus::new();

        assert_eq!(bus.read(VRAM_START), 0);
        assert_eq!(bus.read(VRAM_END), 0);
    }

    #[test]
    fn write_and_read_vram() {
        let mut rng = rand::rng();

        let address: u16 = rng.random_range(VRAM_START..=VRAM_END);
        let value: u8 = rng.random();

        let mut bus = MemoryBus::new();

        bus.write(address, value);

        assert_eq!(bus.read(address), value);
    }

    #[test]
    fn write_and_read_wram() {
        let mut rng = rand::rng();

        let address: u16 = rng.random_range(WRAM_START..=WRAM_END);
        let value: u8 = rng.random();

        let mut bus = MemoryBus::new();

        bus.write(address, value);

        assert_eq!(bus.read(address), value);
    }

    #[test]
    fn vram_and_wram_are_independent() {
        let mut rng = rand::rng();

        let vram_address: u16 = rng.random_range(VRAM_START..=VRAM_END);
        let wram_address: u16 = rng.random_range(WRAM_START..=WRAM_END);

        let vram_value: u8 = rng.random();
        let wram_value: u8 = rng.random();

        let mut bus = MemoryBus::new();

        bus.write(vram_address, vram_value);
        bus.write(wram_address, wram_value);

        assert_eq!(bus.read(vram_address), vram_value);
        assert_eq!(bus.read(wram_address), wram_value);
    }
}
