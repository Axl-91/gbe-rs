use super::*;

fn test_cartridge() -> Cartridge {
    let mbc = CartridgeMbc::Mbc1(Mbc1::new());
    Cartridge::new(vec![0; ROM_BANK_SIZE * 4], RAM_BANK_SIZE, mbc)
}

mod cartridge {
    use super::*;

    #[test]
    fn read_cartridge() {
        let mut rng = rand::rng();
        let mut rom = vec![0; ROM_BANK_SIZE * 4];

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

        let mbc = CartridgeMbc::Mbc1(Mbc1::new());
        let cartridge = Cartridge::new(rom, RAM_BANK_SIZE, mbc);
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

mod dma {
    use super::*;

    const DMA_TRANSFER_BYTES: usize = 160;
    const DMA_SETUP_CYCLES: u16 = 4;
    const DMA_TRANSFER_CYCLES: u16 = 4;

    fn create_bus() -> MemoryBus {
        let rom = vec![0; ROM_BANK_SIZE * 2];
        let mbc = CartridgeMbc::Mbc1(Mbc1::new());
        let cartridge = Cartridge::new(rom, 0, mbc);

        MemoryBus::new(cartridge)
    }

    #[test]
    fn dma_transfers_wram_to_oam() {
        let mut rng = rand::rng();
        let mut bus = create_bus();

        let mut source_data = [0; DMA_TRANSFER_BYTES];

        for value in &mut source_data {
            *value = rng.random();
        }

        for (offset, value) in source_data.iter().enumerate() {
            bus.write(WRAM_START + offset as u16, *value);
        }

        let dma_source = (WRAM_START >> 8) as u8;

        bus.write(DMA_ADDRESS, dma_source);

        // We need to wait 8 T-Cycles until the DMA starts
        let starting_t_cycles = 8;

        for _ in 0..starting_t_cycles {
            bus.tick(1);
        }

        let total_t_cycles = DMA_SETUP_CYCLES + DMA_TRANSFER_BYTES as u16 * DMA_TRANSFER_CYCLES;

        for _ in 0..total_t_cycles {
            bus.tick(1);
        }

        for (offset, expected) in source_data.iter().enumerate() {
            let actual = bus.read(OAM_START + offset as u16);

            assert_eq!(actual, *expected);
        }
    }
}
