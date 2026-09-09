use crate::memory::map::{CARTRIDGE_RAM_END, CARTRIDGE_RAM_START, ROM_BANK_SIZE};

// Addresses ranges for read
const ROM_BANK_0_START: u16 = 0x0000;
const ROM_BANK_0_END: u16 = 0x3FFF;

const ROM_BANK_N_START: u16 = 0x4000;
const ROM_BANK_N_END: u16 = 0x7FFF;

// Addresses ranges for write
pub(crate) const RAM_ENABLE_START: u16 = 0x0000;
pub(crate) const RAM_ENABLE_END: u16 = 0x1FFF;

const ROM_BANK_LOW_START: u16 = 0x2000;
const ROM_BANK_LOW_END: u16 = 0x3FFF;

pub(crate) const BANK_HIGH_START: u16 = 0x4000;
pub(crate) const BANK_HIGH_END: u16 = 0x5FFF;

pub(crate) const BANKING_MODE_START: u16 = 0x6000;
pub(crate) const BANKING_MODE_END: u16 = 0x7FFF;

pub(crate) const RAM_ENABLE_VALUE: u8 = 0x0A;

enum BankingMode {
    Rom,
    Ram,
}

pub struct Mbc1 {
    rom_bank_low: u8,
    bank_high: u8,
    ram_enabled: bool,
    banking_mode: BankingMode,
}

impl Mbc1 {
    pub fn new() -> Self {
        Self {
            rom_bank_low: 1,
            bank_high: 0,
            ram_enabled: false,
            banking_mode: BankingMode::Rom,
        }
    }

    fn get_rom_bank(&self) -> u8 {
        match self.banking_mode {
            BankingMode::Rom => self.rom_bank_low | (self.bank_high << 5),
            BankingMode::Ram => self.rom_bank_low,
        }
    }

    pub fn is_ram_enabled(&self) -> bool {
        self.ram_enabled
    }

    pub fn get_ram_bank(&self) -> u8 {
        match self.banking_mode {
            BankingMode::Rom => 0,
            BankingMode::Ram => self.bank_high,
        }
    }

    pub fn read(&self, address: u16) -> usize {
        match address {
            ROM_BANK_0_START..=ROM_BANK_0_END => address as usize,
            ROM_BANK_N_START..=ROM_BANK_N_END => {
                let offset = address - ROM_BANK_N_START;
                (ROM_BANK_SIZE * self.get_rom_bank() as usize) + offset as usize
            }
            _ => unreachable!("Invalid cartridge address: {address:#06X}"),
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            RAM_ENABLE_START..=RAM_ENABLE_END => {
                // We check for the lower 4 bits
                self.ram_enabled = (value & 0x0F) == RAM_ENABLE_VALUE;
            }

            ROM_BANK_LOW_START..=ROM_BANK_LOW_END => {
                // We check for the lower 5 bits
                self.rom_bank_low = value & 0x1F;
            }

            BANK_HIGH_START..=BANK_HIGH_END => {
                // We check for the lower 2 bits
                self.bank_high = value & 0x03;
            }

            BANKING_MODE_START..=BANKING_MODE_END => {
                // We check for the lower bit
                self.banking_mode = if value & 0x01 == 0 {
                    BankingMode::Rom
                } else {
                    BankingMode::Ram
                };
            }
            CARTRIDGE_RAM_START..=CARTRIDGE_RAM_END => {}
            _ => {}
        }
        if self.rom_bank_low == 0 && self.bank_high == 0 {
            self.rom_bank_low = 1;
        }
    }
}

impl Default for Mbc1 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngExt;

    mod roms {
        use super::*;

        #[test]
        fn read_rom_bank_0() {
            let mut rng = rand::rng();

            let address: u16 = rng.random_range(ROM_BANK_0_START..=ROM_BANK_0_END);

            let mbc = Mbc1::new();

            assert_eq!(mbc.read(address), address as usize);
        }

        #[test]
        #[should_panic]
        fn rejects_unsupported_values() {
            let mut rng = rand::rng();
            let invalid_address = rng.random_range(CARTRIDGE_RAM_END + 1..=0xFFFF);

            let mbc = Mbc1::new();
            mbc.read(invalid_address);
        }

        #[test]
        fn read_rom_bank_n_zero_maps_to_bank_one() {
            let mut rng = rand::rng();

            let address: u16 = rng.random_range(ROM_BANK_N_START..=ROM_BANK_N_END);

            let mut mbc = Mbc1::new();
            mbc.write(ROM_BANK_LOW_START, 0);

            let offset = address - ROM_BANK_N_START;
            let expected = ROM_BANK_SIZE + offset as usize;

            assert_eq!(mbc.read(address), expected);
        }

        #[test]
        fn read_rom_bank_n_initial_bank() {
            let mut rng = rand::rng();

            let address: u16 = rng.random_range(ROM_BANK_N_START..=ROM_BANK_N_END);

            let mbc = Mbc1::new();

            let offset = address - ROM_BANK_N_START;
            let expected = ROM_BANK_SIZE + offset as usize;

            assert_eq!(mbc.read(address), expected);
        }

        #[test]
        fn read_rom_bank_n() {
            let mut rng = rand::rng();

            let address: u16 = rng.random_range(ROM_BANK_N_START..=ROM_BANK_N_END);
            let bank: u8 = rng.random_range(1..=0x1F);

            let mut mbc = Mbc1::new();
            mbc.write(ROM_BANK_LOW_START, bank);

            let offset = address - ROM_BANK_N_START;
            let expected = ROM_BANK_SIZE * bank as usize + offset as usize;

            assert_eq!(mbc.read(address), expected);
        }

        #[test]
        fn read_rom_bank_n_with_high_bits() {
            let mut rng = rand::rng();

            let address: u16 = rng.random_range(ROM_BANK_N_START..=ROM_BANK_N_END);
            let bank_high: u8 = rng.random_range(0..=0x03);
            let bank_low: u8 = rng.random_range(1..=0x1F);

            let mut mbc = Mbc1::new();

            mbc.write(ROM_BANK_LOW_START, bank_low);
            mbc.write(BANK_HIGH_START, bank_high);

            let offset = address - ROM_BANK_N_START;
            let bank = ((bank_high << 5) | bank_low) as usize;
            let expected = ROM_BANK_SIZE * bank + offset as usize;

            assert_eq!(mbc.read(address), expected);
        }

        #[test]
        fn read_rom_bank_n_zero_low_with_high_bits_then_remaps() {
            let mut rng = rand::rng();

            let high: u8 = rng.random_range(1..=0x03);
            let address: u16 = rng.random_range(ROM_BANK_N_START..=ROM_BANK_N_END);

            let mut mbc = Mbc1::new();

            mbc.write(BANK_HIGH_START, high);
            mbc.write(ROM_BANK_LOW_START, 0);

            let offset = address - ROM_BANK_N_START;
            let bank = (high as usize) << 5;
            let expected = ROM_BANK_SIZE * bank + offset as usize;

            // If the low bits are set to 0 while the high bits are non-zero,
            // the selected bank remains determined by the high bits.
            assert_eq!(mbc.read(address), expected);

            mbc.write(BANK_HIGH_START, 0);

            let expected = ROM_BANK_SIZE + offset as usize;

            // Once the high bits are set to 0, the selected bank becomes 0
            // and is remapped to bank 1.
            assert_eq!(mbc.read(address), expected);
        }
    }

    mod ram_enable {
        use super::*;

        #[test]
        fn ram_is_disabled_initially() {
            let mbc = Mbc1::new();

            assert!(!mbc.ram_enabled);
        }

        #[test]
        fn write_ram_enable() {
            let mut rng = rand::rng();

            let upper: u8 = rng.random_range(0..=0x0F);
            let value = (upper << 4) | 0x0A;

            let mut mbc = Mbc1::new();

            mbc.write(RAM_ENABLE_START, value);

            assert!(mbc.ram_enabled);
        }

        #[test]
        fn write_ram_disable() {
            let mut rng = rand::rng();

            let mut value: u8 = rng.random();

            while value & 0x0F == 0x0A {
                value = rng.random();
            }

            let mut mbc = Mbc1::new();

            mbc.write(RAM_ENABLE_START, value);

            assert!(!mbc.ram_enabled);
        }

        #[test]
        fn write_ram_disable_after_write_ram_enable() {
            let mut rng = rand::rng();

            let upper: u8 = rng.random_range(0..=0x0F);
            let enable_value = (upper << 4) | 0x0A;

            let mut disable_value: u8 = rng.random();

            while disable_value & 0x0F == 0x0A {
                disable_value = rng.random();
            }

            let mut mbc = Mbc1::new();

            mbc.write(RAM_ENABLE_START, enable_value);
            assert!(mbc.ram_enabled);

            mbc.write(RAM_ENABLE_START, disable_value);
            assert!(!mbc.ram_enabled);
        }

        #[test]
        fn write_ram_enable_with_all_valid_lower_nibble_values() {
            let mut mbc = Mbc1::new();

            for upper in 0..=0x0F {
                let value = (upper << 4) | 0x0A;

                mbc.write(RAM_ENABLE_START, value);

                assert!(
                    mbc.ram_enabled,
                    "RAM should be enabled with value {value:#04X}"
                );
            }
        }

        #[test]
        fn write_ram_disable_with_all_invalid_lower_nibble_values() {
            let mut mbc = Mbc1::new();

            for upper in 0..=0x0F {
                for lower in 0..=0x0F {
                    if lower == 0x0A {
                        continue;
                    }

                    let value = (upper << 4) | lower;

                    mbc.write(RAM_ENABLE_START, value);

                    assert!(
                        !mbc.ram_enabled,
                        "RAM should be disabled with value {value:#04X}"
                    );
                }
            }
        }
    }

    mod rom_bank {
        use super::*;

        #[test]
        fn read_rom_bank_n_ignores_high_bits_in_ram_banking_mode() {
            let mut rng = rand::rng();

            let high: u8 = rng.random_range(1..=0x03);
            let low: u8 = rng.random_range(1..=0x1F);
            let address: u16 = rng.random_range(ROM_BANK_N_START..=ROM_BANK_N_END);

            let mut mbc = Mbc1::new();

            mbc.write(BANK_HIGH_START, high);
            mbc.write(ROM_BANK_LOW_START, low);
            mbc.write(BANKING_MODE_START, 1);

            let offset = address - ROM_BANK_N_START;
            let expected = ROM_BANK_SIZE * low as usize + offset as usize;

            assert_eq!(mbc.read(address), expected);
        }

        mod low {
            use super::*;

            #[test]
            fn write_rom_bank_low() {
                let mut rng = rand::rng();

                let value: u8 = rng.random_range(1..=0x1F);

                let mut mbc = Mbc1::new();

                mbc.write(ROM_BANK_LOW_START, value);

                assert_eq!(mbc.rom_bank_low, value);
            }

            #[test]
            fn write_rom_bank_low_only_uses_lower_five_bits() {
                let mut rng = rand::rng();

                let value: u8 = rng.random();

                let mut mbc = Mbc1::new();

                mbc.write(ROM_BANK_LOW_START, value);

                let expected = value & 0x1F;

                if expected == 0 {
                    assert_eq!(mbc.rom_bank_low, 1);
                } else {
                    assert_eq!(mbc.rom_bank_low, expected);
                }
            }
        }

        mod high {
            use super::*;

            #[test]
            fn write_bank_high() {
                let mut rng = rand::rng();

                let value: u8 = rng.random_range(0..=0x03);

                let mut mbc = Mbc1::new();

                mbc.write(BANK_HIGH_START, value);

                assert_eq!(mbc.bank_high, value);
            }

            #[test]
            fn write_bank_high_only_uses_lower_two_bits() {
                let mut rng = rand::rng();

                let value: u8 = rng.random();

                let mut mbc = Mbc1::new();

                mbc.write(BANK_HIGH_START, value);

                let expected = value & 0x03;

                assert_eq!(mbc.bank_high, expected);
            }
        }
    }

    mod banking_mode {
        use super::*;

        #[test]
        fn banking_mode_is_rom_initially() {
            let mbc = Mbc1::new();

            assert!(matches!(mbc.banking_mode, BankingMode::Rom));
        }

        #[test]
        fn write_banking_mode_rom() {
            let mut rng = rand::rng();

            let value: u8 = rng.random();

            // Force the lower bit to 0.
            let value = value & !0x01;

            let mut mbc = Mbc1::new();

            mbc.write(BANKING_MODE_START, value);

            assert!(matches!(mbc.banking_mode, BankingMode::Rom));
        }

        #[test]
        fn write_banking_mode_ram() {
            let mut rng = rand::rng();

            let value: u8 = rng.random();

            // Force the lower bit to 1.
            let value = value | 0x01;

            let mut mbc = Mbc1::new();

            mbc.write(BANKING_MODE_START, value);

            assert!(matches!(mbc.banking_mode, BankingMode::Ram));
        }

        #[test]
        fn write_banking_mode_only_uses_lower_bit() {
            let mut rng = rand::rng();

            let upper_bits: u8 = rng.random_range(0..=0x7F);

            let rom_value = upper_bits << 1;
            let ram_value = rom_value | 0x01;

            let mut mbc = Mbc1::new();

            mbc.write(BANKING_MODE_START, rom_value);

            assert!(matches!(mbc.banking_mode, BankingMode::Rom));

            mbc.write(BANKING_MODE_START, ram_value);

            assert!(matches!(mbc.banking_mode, BankingMode::Ram));
        }
    }

    mod ram_bank {
        use super::*;

        #[test]
        fn ram_bank_is_zero_initially() {
            let mbc = Mbc1::new();

            assert_eq!(mbc.get_ram_bank(), 0);
        }

        #[test]
        fn ram_bank_is_zero_in_rom_banking_mode() {
            let mut rng = rand::rng();

            let bank: u8 = rng.random_range(1..=0x03);

            let mut mbc = Mbc1::new();

            mbc.write(BANK_HIGH_START, bank);

            assert!(matches!(mbc.banking_mode, BankingMode::Rom));
            assert_eq!(mbc.get_ram_bank(), 0);
        }

        #[test]
        fn ram_bank_uses_high_bits_in_ram_banking_mode() {
            let mut rng = rand::rng();

            let bank: u8 = rng.random_range(0..=0x03);

            let mut mbc = Mbc1::new();

            mbc.write(BANK_HIGH_START, bank);
            mbc.write(BANKING_MODE_START, 1);

            assert!(matches!(mbc.banking_mode, BankingMode::Ram));
            assert_eq!(mbc.get_ram_bank(), bank);
        }

        #[test]
        fn ram_bank_only_uses_lower_two_bits() {
            let mut rng = rand::rng();

            let upper_bits: u8 = rng.random_range(0..=0x3F);
            let bank = rng.random_range(0..=0x03);

            let value = (upper_bits << 2) | bank;

            let mut mbc = Mbc1::new();

            mbc.write(BANK_HIGH_START, value);
            mbc.write(BANKING_MODE_START, 1);

            assert_eq!(mbc.get_ram_bank(), bank);
        }
    }
}
