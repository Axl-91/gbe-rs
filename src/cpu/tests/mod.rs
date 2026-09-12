use rand::RngExt;

use crate::cartridge::Cartridge;
use crate::memory::MemoryBus;
use crate::memory::map::{ROM_BANK_SIZE, WRAM_END, WRAM_START};

use super::*;

fn create_cpu(opcode: u8, low_value: Option<u8>, high_value: Option<u8>) -> Cpu {
    let mut rom = vec![0; ROM_BANK_SIZE * 2];

    rom[GAME_ENTRY_POINT as usize] = opcode;

    if let Some(value) = low_value {
        rom[GAME_ENTRY_POINT as usize + 1] = value;
    }

    if let Some(value) = high_value {
        rom[GAME_ENTRY_POINT as usize + 2] = value;
    }

    let cartridge = Cartridge::new(rom, 0);
    let bus = MemoryBus::new(cartridge);

    Cpu::new(bus)
}

fn get_rand_wram_address() -> u16 {
    let mut rng = rand::rng();

    // We don't return the first two or last byte of WRAM
    // because some tests also access the previous or next addresses.
    rng.random_range(WRAM_START + 2..WRAM_END)
}

mod adc;
mod add;
mod and;
mod dec;
mod fetch;
mod inc;
mod load;
mod or;
mod sbc;
mod stack;
mod sub;
