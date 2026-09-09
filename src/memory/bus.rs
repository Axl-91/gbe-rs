const CARTRIDGE_ROM_START: u16 = 0x0000;
const CARTRIDGE_ROM_END: u16 = 0x7FFF;

use crate::cartridge::Cartridge;

const VRAM_START: u16 = 0x8000;
const VRAM_END: u16 = 0x9FFF;

const CARTRIDGE_RAM_START: u16 = 0xA000;
const CARTRIDGE_RAM_END: u16 = 0xBFFF;

const WRAM_START: u16 = 0xC000;
const WRAM_END: u16 = 0xDFFF;

pub struct MemoryBus {
    cartridge: Cartridge,
    vram: [u8; 0x2000],
    wram: [u8; 0x2000],
}

impl MemoryBus {
    pub fn new(cartridge: Cartridge) -> Self {
        Self {
            cartridge,
            vram: [0; 0x2000],
            wram: [0; 0x2000],
        }
    }
    pub fn read(&self, address: u16) -> u8 {
        match address {
            CARTRIDGE_ROM_START..=CARTRIDGE_ROM_END => self.cartridge.read(address),

            VRAM_START..=VRAM_END => {
                let offset = (address - VRAM_START) as usize;
                self.vram[offset]
            }

            CARTRIDGE_RAM_START..=CARTRIDGE_RAM_END => self.cartridge.read(address),

            WRAM_START..=WRAM_END => {
                let offset = (address - WRAM_START) as usize;
                self.wram[offset]
            }

            _ => 0,
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            // CARTRIDGE_ROM_START..=CARTRIDGE_ROM_END => self.cartridge.write(address),
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

    fn test_cartridge() -> Cartridge {
        Cartridge::new(vec![0; 0x8000])
    }

    mod cartridge {
        use super::*;

        #[test]
        fn read_cartridge() {
            let mut rng = rand::rng();
            let mut rom = vec![0; 0x8000];

            let addresses: Vec<u16> = vec![
                CARTRIDGE_ROM_START,
                rng.random_range(CARTRIDGE_ROM_START..=CARTRIDGE_ROM_END),
                rng.random_range(CARTRIDGE_ROM_START..=CARTRIDGE_ROM_END),
                rng.random_range(CARTRIDGE_ROM_START..=CARTRIDGE_ROM_END),
                CARTRIDGE_ROM_END,
            ];

            let values: Vec<u8> = vec![
                rng.random(),
                rng.random(),
                rng.random(),
                rng.random(),
                rng.random(),
            ];

            rom[addresses[0] as usize] = values[0];
            rom[addresses[1] as usize] = values[1];
            rom[addresses[2] as usize] = values[2];
            rom[addresses[3] as usize] = values[3];
            rom[addresses[4] as usize] = values[4];

            let cartridge = Cartridge::new(rom);
            let bus = MemoryBus::new(cartridge);

            assert_eq!(bus.read(addresses[0]), values[0]);
            assert_eq!(bus.read(addresses[1]), values[1]);
            assert_eq!(bus.read(addresses[2]), values[2]);
            assert_eq!(bus.read(addresses[3]), values[3]);
            assert_eq!(bus.read(addresses[4]), values[4]);
        }
    }
    mod vram {
        use super::*;

        #[test]
        fn read_vram() {
            let cartridge = test_cartridge();
            let bus = MemoryBus::new(cartridge);

            assert_eq!(bus.read(VRAM_START), 0);
            assert_eq!(bus.read(VRAM_END), 0);
        }

        #[test]
        fn write_and_read_vram() {
            let mut rng = rand::rng();

            let address: u16 = rng.random_range(VRAM_START..=VRAM_END);
            let value: u8 = rng.random();

            let cartridge = test_cartridge();
            let mut bus = MemoryBus::new(cartridge);

            bus.write(address, value);

            assert_eq!(bus.read(address), value);
        }
    }

    mod wram {
        use super::*;
        #[test]
        fn write_and_read_wram() {
            let mut rng = rand::rng();

            let address: u16 = rng.random_range(WRAM_START..=WRAM_END);
            let value: u8 = rng.random();

            let cartridge = test_cartridge();
            let mut bus = MemoryBus::new(cartridge);

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

            let cartridge = test_cartridge();
            let mut bus = MemoryBus::new(cartridge);

            bus.write(vram_address, vram_value);
            bus.write(wram_address, wram_value);

            assert_eq!(bus.read(vram_address), vram_value);
            assert_eq!(bus.read(wram_address), wram_value);
        }
    }
}
