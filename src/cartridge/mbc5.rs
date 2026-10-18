//! MBC5 memory bank controller implementation.
//!
//! Handles ROM and RAM bank selection for Game Boy cartridges.

use crate::{
    cartridge::mbc::Mbc,
    memory::map::{CARTRIDGE_RAM_START, RAM_BANK_SIZE, ROM_BANK_SIZE},
};

const FIXED_ROM_BANK_START: u16 = 0x0000;
const FIXED_ROM_BANK_END: u16 = 0x3FFF;

const SWITCHABLE_ROM_BANK_START: u16 = 0x4000;
const SWITCHABLE_ROM_BANK_END: u16 = 0x7FFF;

const RAM_ENABLE_START: u16 = 0x0000;
const RAM_ENABLE_END: u16 = 0x1FFF;

const ROM_BANK_LOW_START: u16 = 0x2000;
const ROM_BANK_LOW_END: u16 = 0x2FFF;

const ROM_BANK_HIGH_START: u16 = 0x3000;
const ROM_BANK_HIGH_END: u16 = 0x3FFF;

const RAM_BANK_SELECT_START: u16 = 0x4000;
const RAM_BANK_SELECT_END: u16 = 0x5FFF;

const RAM_ADDRESS_END: u16 = 0xBFFF;

const RAM_ENABLE_VALUE: u8 = 0x0A;
const ROM_BANK_LOW_MASK: u16 = 0x00FF;
const ROM_BANK_HIGH_MASK: u8 = 0x01;
const RAM_BANK_MASK: u8 = 0x0F;

const DEFAULT_ROM_BANK: u16 = 1;
const DEFAULT_RAM_BANK: u8 = 0;
const DISABLED_RAM_VALUE: u8 = 0xFF;

/// Represents an MBC5 memory bank controller.
pub struct Mbc5 {
    rom_bank: u16,
    ram_bank: u8,
    ram_enabled: bool,
    ram: Vec<u8>,
}

impl Mbc5 {
    /// Creates a new MBC5 with ROM bank 1 selected and RAM disabled.
    pub fn new(ram_size: usize) -> Self {
        Self {
            rom_bank: DEFAULT_ROM_BANK,
            ram_bank: DEFAULT_RAM_BANK,
            ram_enabled: false,
            ram: vec![0; ram_size],
        }
    }

    fn get_rom_bank_offset(&self) -> usize {
        ROM_BANK_SIZE * self.rom_bank as usize
    }

    fn get_ram_offset(&self, address: u16) -> Option<usize> {
        if self.ram.is_empty() {
            return None;
        }

        let address_offset = (address - CARTRIDGE_RAM_START) as usize;
        let bank_offset = self.ram_bank as usize * RAM_BANK_SIZE;
        let ram_address = bank_offset + address_offset;

        (ram_address < self.ram.len()).then_some(ram_address)
    }
}

impl Mbc for Mbc5 {
    fn read(&self, address: u16) -> usize {
        match address {
            FIXED_ROM_BANK_START..=FIXED_ROM_BANK_END => address as usize,
            SWITCHABLE_ROM_BANK_START..=SWITCHABLE_ROM_BANK_END => {
                let offset = address - SWITCHABLE_ROM_BANK_START;
                self.get_rom_bank_offset() + offset as usize
            }
            _ => unreachable!("Invalid cartridge address: {address:#06X}"),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            RAM_ENABLE_START..=RAM_ENABLE_END => {
                self.ram_enabled = value == RAM_ENABLE_VALUE;
            }
            ROM_BANK_LOW_START..=ROM_BANK_LOW_END => {
                self.rom_bank = (self.rom_bank & !ROM_BANK_LOW_MASK) | value as u16;
            }
            ROM_BANK_HIGH_START..=ROM_BANK_HIGH_END => {
                let high_bit = (value & ROM_BANK_HIGH_MASK) as u16;
                self.rom_bank = (self.rom_bank & ROM_BANK_LOW_MASK) | (high_bit << 8);
            }
            RAM_BANK_SELECT_START..=RAM_BANK_SELECT_END => {
                self.ram_bank = value & RAM_BANK_MASK;
            }
            _ => {}
        }
    }

    fn read_ram(&self, address: u16) -> u8 {
        if !self.ram_enabled || !(CARTRIDGE_RAM_START..=RAM_ADDRESS_END).contains(&address) {
            return DISABLED_RAM_VALUE;
        }

        match self.get_ram_offset(address) {
            Some(offset) => self.ram[offset],
            None => DISABLED_RAM_VALUE,
        }
    }

    fn write_ram(&mut self, address: u16, value: u8) {
        if !self.ram_enabled || !(CARTRIDGE_RAM_START..=RAM_ADDRESS_END).contains(&address) {
            return;
        }

        if let Some(offset) = self.get_ram_offset(address) {
            self.ram[offset] = value;
        }
    }

    fn is_ram_enabled(&self) -> bool {
        self.ram_enabled
    }

    fn get_ram_bank(&self) -> u8 {
        self.ram_bank
    }
}

impl Default for Mbc5 {
    fn default() -> Self {
        Self::new(RAM_BANK_SIZE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cartridge::mbc::Mbc;

    const FIRST_ROM_BANK: u16 = 1;
    const SECOND_ROM_BANK: u16 = 2;
    const ROM_BANK_ZERO: u16 = 0;
    const ROM_BANK_LOW_MAX: u8 = 0xFF;
    const ROM_BANK_HIGH_BIT: u8 = 1;
    const ROM_BANK_HIGH_BITS_MASK: u8 = 0xFE;

    const FIRST_RAM_BANK: u8 = 0;
    const SECOND_RAM_BANK: u8 = 1;
    const FOURTH_RAM_BANK: u8 = 3;
    const INVALID_RAM_BANK: u8 = 4;

    const RAM_ENABLE_COMMAND: u8 = 0x0A;
    const RAM_DISABLE_COMMAND: u8 = 0x00;
    const RAM_DISABLED_VALUE: u8 = 0xFF;

    const FIRST_TEST_VALUE: u8 = 0x12;
    const SECOND_TEST_VALUE: u8 = 0x34;

    fn create_mbc5_with_four_ram_banks() -> Mbc5 {
        Mbc5::new(RAM_BANK_SIZE * 4)
    }

    fn enable_ram(mbc: &mut Mbc5) {
        mbc.write(RAM_ENABLE_START, RAM_ENABLE_COMMAND);
    }

    #[test]
    fn fixed_rom_window_maps_bank_zero() {
        let mbc = Mbc5::new(0);

        assert_eq!(mbc.read(FIXED_ROM_BANK_START), 0);
        assert_eq!(mbc.read(FIXED_ROM_BANK_END), ROM_BANK_SIZE - 1);
    }

    #[test]
    fn switchable_rom_window_defaults_to_bank_one() {
        let mbc = Mbc5::new(0);

        assert_eq!(
            mbc.read(SWITCHABLE_ROM_BANK_START),
            ROM_BANK_SIZE * FIRST_ROM_BANK as usize
        );
    }

    #[test]
    fn selecting_low_rom_bank_changes_switchable_window() {
        let mut mbc = Mbc5::new(0);

        mbc.write(ROM_BANK_LOW_START, SECOND_ROM_BANK as u8);

        assert_eq!(
            mbc.read(SWITCHABLE_ROM_BANK_START),
            ROM_BANK_SIZE * SECOND_ROM_BANK as usize
        );
    }

    #[test]
    fn rom_bank_zero_is_selectable() {
        let mut mbc = Mbc5::new(0);

        mbc.write(ROM_BANK_LOW_START, ROM_BANK_ZERO as u8);
        mbc.write(ROM_BANK_HIGH_START, 0);

        assert_eq!(mbc.read(SWITCHABLE_ROM_BANK_START), 0);
    }

    #[test]
    fn high_rom_bank_bit_selects_bank_256() {
        let mut mbc = Mbc5::new(0);

        mbc.write(ROM_BANK_LOW_START, ROM_BANK_ZERO as u8);
        mbc.write(ROM_BANK_HIGH_START, ROM_BANK_HIGH_BIT);

        assert_eq!(mbc.read(SWITCHABLE_ROM_BANK_START), ROM_BANK_SIZE * 256);
    }

    #[test]
    fn changing_low_rom_bank_preserves_high_bit() {
        let mut mbc = Mbc5::new(0);

        mbc.write(ROM_BANK_HIGH_START, ROM_BANK_HIGH_BIT);
        mbc.write(ROM_BANK_LOW_START, SECOND_ROM_BANK as u8);

        let expected_bank = 256 + SECOND_ROM_BANK as usize;

        assert_eq!(
            mbc.read(SWITCHABLE_ROM_BANK_START),
            ROM_BANK_SIZE * expected_bank
        );
    }

    #[test]
    fn changing_high_rom_bank_preserves_low_bits() {
        let mut mbc = Mbc5::new(0);

        mbc.write(ROM_BANK_LOW_START, ROM_BANK_LOW_MAX);
        mbc.write(ROM_BANK_HIGH_START, ROM_BANK_HIGH_BIT);

        let expected_bank = 256 + ROM_BANK_LOW_MAX as usize;

        assert_eq!(
            mbc.read(SWITCHABLE_ROM_BANK_START),
            ROM_BANK_SIZE * expected_bank
        );
    }

    #[test]
    fn rom_bank_high_register_uses_only_one_bit() {
        let mut mbc = Mbc5::new(0);

        mbc.write(ROM_BANK_LOW_START, ROM_BANK_ZERO as u8);
        mbc.write(ROM_BANK_HIGH_START, ROM_BANK_HIGH_BITS_MASK);

        assert_eq!(mbc.read(SWITCHABLE_ROM_BANK_START), 0);
    }

    #[test]
    fn ram_is_disabled_by_default() {
        let mbc = create_mbc5_with_four_ram_banks();

        assert!(!mbc.is_ram_enabled());
        assert_eq!(mbc.read_ram(CARTRIDGE_RAM_START), RAM_DISABLED_VALUE);
    }

    #[test]
    fn writing_ram_while_disabled_has_no_effect() {
        let mut mbc = create_mbc5_with_four_ram_banks();

        mbc.write_ram(CARTRIDGE_RAM_START, FIRST_TEST_VALUE);
        enable_ram(&mut mbc);

        assert_eq!(mbc.read_ram(CARTRIDGE_RAM_START), 0);
    }

    #[test]
    fn enabling_ram_allows_reading_and_writing() {
        let mut mbc = create_mbc5_with_four_ram_banks();

        enable_ram(&mut mbc);
        mbc.write_ram(CARTRIDGE_RAM_START, FIRST_TEST_VALUE);

        assert!(mbc.is_ram_enabled());
        assert_eq!(mbc.read_ram(CARTRIDGE_RAM_START), FIRST_TEST_VALUE);
    }

    #[test]
    fn disabling_ram_prevents_access() {
        let mut mbc = create_mbc5_with_four_ram_banks();

        enable_ram(&mut mbc);
        mbc.write_ram(CARTRIDGE_RAM_START, FIRST_TEST_VALUE);
        mbc.write(RAM_ENABLE_START, RAM_DISABLE_COMMAND);

        assert!(!mbc.is_ram_enabled());
        assert_eq!(mbc.read_ram(CARTRIDGE_RAM_START), RAM_DISABLED_VALUE);
    }

    #[test]
    fn ram_banks_are_independent() {
        let mut mbc = create_mbc5_with_four_ram_banks();

        enable_ram(&mut mbc);

        mbc.write(RAM_BANK_SELECT_START, FIRST_RAM_BANK);
        mbc.write_ram(CARTRIDGE_RAM_START, FIRST_TEST_VALUE);

        mbc.write(RAM_BANK_SELECT_START, SECOND_RAM_BANK);
        mbc.write_ram(CARTRIDGE_RAM_START, SECOND_TEST_VALUE);

        mbc.write(RAM_BANK_SELECT_START, FIRST_RAM_BANK);
        assert_eq!(mbc.read_ram(CARTRIDGE_RAM_START), FIRST_TEST_VALUE);

        mbc.write(RAM_BANK_SELECT_START, SECOND_RAM_BANK);
        assert_eq!(mbc.read_ram(CARTRIDGE_RAM_START), SECOND_TEST_VALUE);
    }

    #[test]
    fn invalid_ram_bank_does_not_access_ram() {
        let mut mbc = create_mbc5_with_four_ram_banks();

        enable_ram(&mut mbc);
        mbc.write(RAM_BANK_SELECT_START, INVALID_RAM_BANK);
        mbc.write_ram(CARTRIDGE_RAM_START, FIRST_TEST_VALUE);

        assert_eq!(mbc.read_ram(CARTRIDGE_RAM_START), RAM_DISABLED_VALUE);
    }

    #[test]
    fn ram_bank_selection_is_reported() {
        let mut mbc = create_mbc5_with_four_ram_banks();

        mbc.write(RAM_BANK_SELECT_START, FOURTH_RAM_BANK);

        assert_eq!(mbc.get_ram_bank(), FOURTH_RAM_BANK);
    }

    #[test]
    fn default_starts_with_rom_bank_one_and_ram_disabled() {
        let mbc = Mbc5::default();

        assert_eq!(
            mbc.read(SWITCHABLE_ROM_BANK_START),
            ROM_BANK_SIZE * FIRST_ROM_BANK as usize
        );
        assert!(!mbc.is_ram_enabled());
    }
}
