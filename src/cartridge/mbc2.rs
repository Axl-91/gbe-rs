//! MBC2 memory bank controller implementation.
//!
//! Handles ROM banking and the internal 4-bit RAM of MBC2 cartridges.

use crate::{cartridge::mbc::Mbc, memory::map::ROM_BANK_SIZE};

const FIXED_ROM_BANK_START: u16 = 0x0000;
const FIXED_ROM_BANK_END: u16 = 0x3FFF;

const SWITCHABLE_ROM_BANK_START: u16 = 0x4000;
const SWITCHABLE_ROM_BANK_END: u16 = 0x7FFF;

const CONTROL_START: u16 = 0x0000;
const CONTROL_END: u16 = 0x3FFF;
const ADDRESS_BIT_8: u16 = 0x0100;

const RAM_ENABLE_VALUE: u8 = 0x0A;

const ROM_BANK_MASK: u8 = 0x0F;
const RAM_ADDRESS_MASK: u16 = 0x01FF;
const RAM_VALUE_MASK: u8 = 0x0F;
const RAM_READ_MASK: u8 = 0xF0;

const DEFAULT_ROM_BANK: u8 = 1;
const RAM_SIZE: usize = 0x0200;

/// Represents an MBC2 memory bank controller.
pub struct Mbc2 {
    has_battery: bool,
    rom_bank: u8,
    ram_enabled: bool,
    ram: [u8; RAM_SIZE],
}

impl Mbc2 {
    /// Creates a new MBC2 with RAM disabled and ROM bank 1 selected.
    pub fn new(has_battery: bool) -> Self {
        Self {
            has_battery,
            rom_bank: DEFAULT_ROM_BANK,
            ram_enabled: false,
            ram: [0; RAM_SIZE],
        }
    }

    fn get_rom_bank_offset(&self) -> usize {
        ROM_BANK_SIZE * self.rom_bank as usize
    }
}

impl Mbc for Mbc2 {
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

    /// Updates the MBC2 state according to a cartridge control write.
    fn write(&mut self, address: u16, value: u8) {
        match address {
            CONTROL_START..=CONTROL_END if address & ADDRESS_BIT_8 == 0 => {
                self.ram_enabled = (value & 0x0F) == RAM_ENABLE_VALUE;
            }

            CONTROL_START..=CONTROL_END if address & ADDRESS_BIT_8 != 0 => {
                self.write_rom_bank(value);
            }

            _ => {}
        }
    }

    fn read_ram(&self, address: u16) -> u8 {
        if !self.ram_enabled {
            return 0xFF;
        }

        let index = (address & RAM_ADDRESS_MASK) as usize;
        RAM_READ_MASK | (self.ram[index] & RAM_VALUE_MASK)
    }

    fn write_ram(&mut self, address: u16, value: u8) {
        if !self.ram_enabled {
            return;
        }

        let index = (address & RAM_ADDRESS_MASK) as usize;
        self.ram[index] = value & RAM_VALUE_MASK;
    }

    fn has_battery(&self) -> bool {
        self.has_battery
    }

    fn is_ram_enabled(&self) -> bool {
        self.ram_enabled
    }

    fn get_ram_bank(&self) -> u8 {
        0
    }

    fn get_save_data(&self) -> Option<Vec<u8>> {
        if !self.has_battery || self.ram.is_empty() {
            return None;
        }

        Some(self.ram.to_vec().clone())
    }

    fn load_save_data(&mut self, data: &[u8]) -> bool {
        if !self.has_battery || data.len() != self.ram.len() {
            return false;
        }

        self.ram.copy_from_slice(data);
        true
    }
}

impl Mbc2 {
    fn write_rom_bank(&mut self, value: u8) {
        let bank = value & ROM_BANK_MASK;

        self.rom_bank = if bank == 0 { DEFAULT_ROM_BANK } else { bank };
    }
}

impl Default for Mbc2 {
    fn default() -> Self {
        Self::new(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXED_BANK_FIRST_ADDRESS: u16 = 0x0000;
    const FIXED_BANK_LAST_ADDRESS: u16 = 0x3FFF;

    const SWITCHABLE_BANK_FIRST_ADDRESS: u16 = 0x4000;

    const RAM_FIRST_ADDRESS: u16 = 0xA000;
    const RAM_ALIASED_ADDRESS: u16 = 0xA200;

    const RAM_ENABLE_ADDRESS: u16 = 0x0000;
    const ROM_BANK_SELECT_ADDRESS: u16 = 0x0100;

    const RAM_ENABLE_VALUE: u8 = 0x0A;
    const RAM_DISABLE_VALUE: u8 = 0x00;

    const FIRST_SWITCHABLE_BANK: u8 = 1;
    const SECOND_SWITCHABLE_BANK: u8 = 2;
    const THIRD_SWITCHABLE_BANK: u8 = 3;

    #[test]
    fn fixed_rom_bank_maps_addresses_directly() {
        let mbc = Mbc2::default();

        assert_eq!(mbc.read(FIXED_BANK_FIRST_ADDRESS), 0);
        assert_eq!(
            mbc.read(FIXED_BANK_LAST_ADDRESS),
            FIXED_BANK_LAST_ADDRESS as usize
        );
    }

    #[test]
    fn switchable_rom_bank_defaults_to_bank_one() {
        let mbc = Mbc2::default();

        assert_eq!(
            mbc.read(SWITCHABLE_BANK_FIRST_ADDRESS),
            ROM_BANK_SIZE * FIRST_SWITCHABLE_BANK as usize
        );
    }

    #[test]
    fn writing_rom_bank_selects_the_requested_bank() {
        let mut mbc = Mbc2::default();

        mbc.write(ROM_BANK_SELECT_ADDRESS, THIRD_SWITCHABLE_BANK);

        assert_eq!(
            mbc.read(SWITCHABLE_BANK_FIRST_ADDRESS),
            ROM_BANK_SIZE * THIRD_SWITCHABLE_BANK as usize
        );
    }

    #[test]
    fn selecting_rom_bank_zero_selects_bank_one() {
        let mut mbc = Mbc2::default();

        mbc.write(ROM_BANK_SELECT_ADDRESS, 0);

        assert_eq!(
            mbc.read(SWITCHABLE_BANK_FIRST_ADDRESS),
            ROM_BANK_SIZE * FIRST_SWITCHABLE_BANK as usize
        );
    }

    #[test]
    fn rom_bank_selection_uses_only_the_lower_four_bits() {
        let mut mbc = Mbc2::default();
        let value_with_upper_bits = 0xF0 | SECOND_SWITCHABLE_BANK;

        mbc.write(ROM_BANK_SELECT_ADDRESS, value_with_upper_bits);

        assert_eq!(
            mbc.read(SWITCHABLE_BANK_FIRST_ADDRESS),
            ROM_BANK_SIZE * SECOND_SWITCHABLE_BANK as usize
        );
    }

    #[test]
    fn ram_is_disabled_by_default() {
        let mbc = Mbc2::default();

        assert!(!mbc.is_ram_enabled());
        assert_eq!(mbc.read_ram(RAM_FIRST_ADDRESS), 0xFF);
    }

    #[test]
    fn writing_ram_enable_value_enables_ram() {
        let mut mbc = Mbc2::default();

        mbc.write(RAM_ENABLE_ADDRESS, RAM_ENABLE_VALUE);

        assert!(mbc.is_ram_enabled());
    }

    #[test]
    fn writing_ram_disable_value_disables_ram() {
        let mut mbc = Mbc2::default();

        mbc.write(RAM_ENABLE_ADDRESS, RAM_ENABLE_VALUE);
        mbc.write(RAM_ENABLE_ADDRESS, RAM_DISABLE_VALUE);

        assert!(!mbc.is_ram_enabled());
        assert_eq!(mbc.read_ram(RAM_FIRST_ADDRESS), 0xFF);
    }

    #[test]
    fn ram_writes_are_ignored_when_ram_is_disabled() {
        let mut mbc = Mbc2::default();

        mbc.write_ram(RAM_FIRST_ADDRESS, 0x05);

        mbc.write(RAM_ENABLE_ADDRESS, RAM_ENABLE_VALUE);

        assert_eq!(mbc.read_ram(RAM_FIRST_ADDRESS), 0xF0);
    }

    #[test]
    fn ram_stores_only_the_lower_four_bits() {
        let mut mbc = Mbc2::default();

        mbc.write(RAM_ENABLE_ADDRESS, RAM_ENABLE_VALUE);
        mbc.write_ram(RAM_FIRST_ADDRESS, 0xAB);

        assert_eq!(mbc.read_ram(RAM_FIRST_ADDRESS), 0xFB);
    }

    #[test]
    fn ram_reads_set_the_upper_four_bits() {
        let mut mbc = Mbc2::default();

        mbc.write(RAM_ENABLE_ADDRESS, RAM_ENABLE_VALUE);
        mbc.write_ram(RAM_FIRST_ADDRESS, 0x05);

        assert_eq!(mbc.read_ram(RAM_FIRST_ADDRESS), 0xF5);
    }

    #[test]
    fn ram_addresses_mirror_every_512_bytes() {
        let mut mbc = Mbc2::default();

        mbc.write(RAM_ENABLE_ADDRESS, RAM_ENABLE_VALUE);
        mbc.write_ram(RAM_FIRST_ADDRESS, 0x07);

        assert_eq!(mbc.read_ram(RAM_ALIASED_ADDRESS), 0xF7);
    }

    #[test]
    fn ram_enable_and_rom_bank_selection_use_address_bit_eight() {
        let mut mbc = Mbc2::default();

        // A8 = 0: enables RAM.
        mbc.write(RAM_ENABLE_ADDRESS, RAM_ENABLE_VALUE);
        assert!(mbc.is_ram_enabled());

        // A8 = 1: selects a ROM bank without disabling RAM.
        mbc.write(ROM_BANK_SELECT_ADDRESS, THIRD_SWITCHABLE_BANK);

        assert!(mbc.is_ram_enabled());
        assert_eq!(
            mbc.read(SWITCHABLE_BANK_FIRST_ADDRESS),
            ROM_BANK_SIZE * THIRD_SWITCHABLE_BANK as usize
        );
    }

    #[test]
    fn addresses_with_a8_clear_enable_ram() {
        let mut mbc = Mbc2::default();
        let ram_control_address: u16 = 0x0200;

        mbc.write(ram_control_address, RAM_ENABLE_VALUE);

        assert!(mbc.is_ram_enabled());
    }

    #[test]
    fn ram_banks_are_not_switchable() {
        let mbc = Mbc2::default();

        assert_eq!(mbc.get_ram_bank(), 0);
    }

    #[test]
    fn default_creates_the_initial_mbc2_state() {
        let mbc = Mbc2::default();

        assert!(!mbc.is_ram_enabled());
        assert_eq!(
            mbc.read(SWITCHABLE_BANK_FIRST_ADDRESS),
            ROM_BANK_SIZE * FIRST_SWITCHABLE_BANK as usize
        );
        assert_eq!(mbc.read_ram(RAM_FIRST_ADDRESS), 0xFF);
    }
}
