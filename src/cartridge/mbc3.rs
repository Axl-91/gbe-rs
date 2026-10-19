//! MBC3 memory bank controller implementation.
//!
//! Handles ROM banking and external RAM banking.
//! RTC support is not implemented yet.

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

const ROM_BANK_SELECT_START: u16 = 0x2000;
const ROM_BANK_SELECT_END: u16 = 0x3FFF;

const RAM_BANK_SELECT_START: u16 = 0x4000;
const RAM_BANK_SELECT_END: u16 = 0x5FFF;

const RAM_ENABLE_VALUE: u8 = 0x0A;
const ROM_BANK_MASK: u8 = 0x7F;

const DEFAULT_ROM_BANK: u8 = 1;
const DEFAULT_RAM_BANK: u8 = 0;

const LAST_RAM_BANK: u8 = 3;

const RAM_ADDRESS_END: u16 = 0xBFFF;
const DISABLED_RAM_VALUE: u8 = 0xFF;

/// Represents an MBC3 memory bank controller.
pub struct Mbc3 {
    has_battery: bool,
    rom_bank: u8,
    ram_bank: u8,
    ram_enabled: bool,
    ram: Vec<u8>,
}

impl Mbc3 {
    /// Creates an MBC3 with the RAM size specified by the cartridge header.
    pub fn new(has_battery: bool, ram_size: usize) -> Self {
        Self {
            has_battery,
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
        if self.ram.is_empty() || self.ram_bank > LAST_RAM_BANK {
            return None;
        }

        let address_offset = (address - CARTRIDGE_RAM_START) as usize;
        let bank_offset = self.ram_bank as usize * RAM_BANK_SIZE;
        let ram_address = bank_offset + address_offset;

        (ram_address < self.ram.len()).then_some(ram_address)
    }
}

impl Mbc for Mbc3 {
    /// Translates a cartridge address into a physical ROM address.
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

    /// Updates the MBC3 registers.
    fn write(&mut self, address: u16, value: u8) {
        match address {
            RAM_ENABLE_START..=RAM_ENABLE_END => {
                self.ram_enabled = (value & 0x0F) == RAM_ENABLE_VALUE;
            }

            ROM_BANK_SELECT_START..=ROM_BANK_SELECT_END => {
                let bank = value & ROM_BANK_MASK;

                self.rom_bank = if bank == 0 { DEFAULT_ROM_BANK } else { bank };
            }

            RAM_BANK_SELECT_START..=RAM_BANK_SELECT_END => {
                self.ram_bank = value;
            }

            // RTC latch writes are not implemented yet.
            0x6000..=0x7FFF => {}

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

    fn has_battery(&self) -> bool {
        self.has_battery
    }

    fn is_ram_enabled(&self) -> bool {
        self.ram_enabled
    }

    fn get_ram_bank(&self) -> u8 {
        self.ram_bank
    }

    fn get_save_data(&self) -> Option<Vec<u8>> {
        if !self.has_battery || self.ram.is_empty() {
            return None;
        }

        Some(self.ram.clone())
    }

    fn load_save_data(&mut self, data: &[u8]) -> bool {
        if !self.has_battery || data.len() != self.ram.len() {
            return false;
        }

        self.ram.copy_from_slice(data);
        true
    }
}

impl Default for Mbc3 {
    fn default() -> Self {
        Self::new(false, RAM_BANK_SIZE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cartridge::mbc::Mbc;
    use crate::memory::map::CARTRIDGE_RAM_START;

    const ROM_BANK_ZERO: u8 = 0;
    const FIRST_SWITCHABLE_ROM_BANK: u8 = 1;
    const SECOND_SWITCHABLE_ROM_BANK: u8 = 2;
    const LAST_ROM_BANK: u8 = 0x7F;
    const INVALID_ROM_BANK_BITS: u8 = 0x80;

    const RAM_BANK_ZERO: u8 = 0;
    const RAM_BANK_ONE: u8 = 1;
    const RAM_BANK_TWO: u8 = 2;
    const INVALID_RAM_BANK: u8 = 4;

    const RAM_DISABLED_VALUE: u8 = 0xFF;
    const RAM_ENABLE_COMMAND: u8 = 0x0A;
    const RAM_DISABLE_COMMAND: u8 = 0x00;

    const ROM_BANK_SELECT_ADDRESS: u16 = 0x2000;
    const RAM_BANK_SELECT_ADDRESS: u16 = 0x4000;
    const RAM_ENABLE_ADDRESS: u16 = 0x0000;

    fn create_mbc3_with_four_ram_banks() -> Mbc3 {
        Mbc3::new(false, RAM_BANK_SIZE * 4)
    }

    fn enable_ram(mbc: &mut Mbc3) {
        mbc.write(RAM_ENABLE_ADDRESS, RAM_ENABLE_COMMAND);
    }

    #[test]
    fn fixed_rom_bank_always_maps_bank_zero() {
        let mbc = Mbc3::new(false, 0);

        assert_eq!(mbc.read(0x0000), 0);
        assert_eq!(mbc.read(0x3FFF), ROM_BANK_SIZE - 1);
    }

    #[test]
    fn switchable_rom_bank_defaults_to_bank_one() {
        let mbc = Mbc3::new(false, 0);

        assert_eq!(
            mbc.read(0x4000),
            ROM_BANK_SIZE * FIRST_SWITCHABLE_ROM_BANK as usize
        );
    }

    #[test]
    fn selecting_rom_bank_changes_switchable_rom_window() {
        let mut mbc = Mbc3::new(false, 0);

        mbc.write(ROM_BANK_SELECT_ADDRESS, SECOND_SWITCHABLE_ROM_BANK);

        assert_eq!(
            mbc.read(0x4000),
            ROM_BANK_SIZE * SECOND_SWITCHABLE_ROM_BANK as usize
        );
    }

    #[test]
    fn selecting_rom_bank_zero_maps_bank_one() {
        let mut mbc = Mbc3::new(false, 0);

        mbc.write(ROM_BANK_SELECT_ADDRESS, ROM_BANK_ZERO);

        assert_eq!(
            mbc.read(0x4000),
            ROM_BANK_SIZE * FIRST_SWITCHABLE_ROM_BANK as usize
        );
    }

    #[test]
    fn rom_bank_selection_uses_seven_bits() {
        let mut mbc = Mbc3::new(false, 0);
        let selected_value = LAST_ROM_BANK | INVALID_ROM_BANK_BITS;

        mbc.write(ROM_BANK_SELECT_ADDRESS, selected_value);

        assert_eq!(mbc.read(0x4000), ROM_BANK_SIZE * LAST_ROM_BANK as usize);
    }

    #[test]
    fn ram_is_disabled_by_default() {
        let mbc = create_mbc3_with_four_ram_banks();

        assert!(!mbc.is_ram_enabled());
        assert_eq!(mbc.read_ram(CARTRIDGE_RAM_START), RAM_DISABLED_VALUE);
    }

    #[test]
    fn writing_ram_while_disabled_has_no_effect() {
        let mut mbc = create_mbc3_with_four_ram_banks();

        mbc.write_ram(CARTRIDGE_RAM_START, 0x42);

        enable_ram(&mut mbc);

        assert_eq!(mbc.read_ram(CARTRIDGE_RAM_START), 0);
    }

    #[test]
    fn enabling_ram_allows_reading_and_writing() {
        let mut mbc = create_mbc3_with_four_ram_banks();
        let expected_value = 0x42;

        enable_ram(&mut mbc);
        mbc.write_ram(CARTRIDGE_RAM_START, expected_value);

        assert!(mbc.is_ram_enabled());
        assert_eq!(mbc.read_ram(CARTRIDGE_RAM_START), expected_value);
    }

    #[test]
    fn disabling_ram_prevents_access_again() {
        let mut mbc = create_mbc3_with_four_ram_banks();
        let expected_value = 0x42;

        enable_ram(&mut mbc);
        mbc.write_ram(CARTRIDGE_RAM_START, expected_value);

        mbc.write(RAM_ENABLE_ADDRESS, RAM_DISABLE_COMMAND);

        assert!(!mbc.is_ram_enabled());
        assert_eq!(mbc.read_ram(CARTRIDGE_RAM_START), RAM_DISABLED_VALUE);
    }

    #[test]
    fn ram_banks_are_independent() {
        let mut mbc = create_mbc3_with_four_ram_banks();
        let bank_zero_value = 0x12;
        let bank_one_value = 0x34;

        enable_ram(&mut mbc);

        mbc.write(RAM_BANK_SELECT_ADDRESS, RAM_BANK_ZERO);
        mbc.write_ram(CARTRIDGE_RAM_START, bank_zero_value);

        mbc.write(RAM_BANK_SELECT_ADDRESS, RAM_BANK_ONE);
        mbc.write_ram(CARTRIDGE_RAM_START, bank_one_value);

        mbc.write(RAM_BANK_SELECT_ADDRESS, RAM_BANK_ZERO);
        assert_eq!(mbc.read_ram(CARTRIDGE_RAM_START), bank_zero_value);

        mbc.write(RAM_BANK_SELECT_ADDRESS, RAM_BANK_ONE);
        assert_eq!(mbc.read_ram(CARTRIDGE_RAM_START), bank_one_value);
    }

    #[test]
    fn invalid_ram_bank_does_not_access_ram() {
        let mut mbc = create_mbc3_with_four_ram_banks();

        enable_ram(&mut mbc);
        mbc.write(RAM_BANK_SELECT_ADDRESS, INVALID_RAM_BANK);
        mbc.write_ram(CARTRIDGE_RAM_START, 0x42);

        assert_eq!(mbc.read_ram(CARTRIDGE_RAM_START), RAM_DISABLED_VALUE);
    }

    #[test]
    fn ram_bank_selection_is_reported() {
        let mut mbc = create_mbc3_with_four_ram_banks();

        mbc.write(RAM_BANK_SELECT_ADDRESS, RAM_BANK_TWO);

        assert_eq!(mbc.get_ram_bank(), RAM_BANK_TWO);
    }

    #[test]
    fn default_starts_with_rom_bank_one_and_ram_disabled() {
        let mbc = Mbc3::default();

        assert_eq!(
            mbc.read(0x4000),
            ROM_BANK_SIZE * FIRST_SWITCHABLE_ROM_BANK as usize
        );
        assert!(!mbc.is_ram_enabled());
    }
}
