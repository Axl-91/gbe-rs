use rand::RngExt;

use crate::cartridge::Cartridge;
use crate::cartridge::mbc::CartridgeMbc;
use crate::cartridge::mbc1::Mbc1;
use crate::memory::MemoryBus;
use crate::memory::map::{ROM_BANK_SIZE, WRAM_END, WRAM_START};

use super::*;

/// Address where the Game Boy starts executing the game after the boot sequence.
const GAME_ENTRY_POINT: u16 = 0x0100;

fn create_cpu(opcode: u8, low_value: Option<u8>, high_value: Option<u8>) -> Cpu {
    let mut rom = vec![0; ROM_BANK_SIZE * 2];

    rom[GAME_ENTRY_POINT as usize] = opcode;

    if let Some(value) = low_value {
        rom[GAME_ENTRY_POINT as usize + 1] = value;
    }

    if let Some(value) = high_value {
        rom[GAME_ENTRY_POINT as usize + 2] = value;
    }

    let mbc = CartridgeMbc::Mbc1(Mbc1::new());
    let cartridge = Cartridge::new(rom, 0, mbc);
    let bus = MemoryBus::new(cartridge);

    Cpu::new(bus)
}

fn get_rand_wram_address() -> u16 {
    let mut rng = rand::rng();

    // We don't return the first two or last byte of WRAM
    // because some tests also access the previous or next addresses.
    rng.random_range(WRAM_START + 2..WRAM_END)
}

mod arithmetic;
mod cb;
mod control;
mod fetch;
mod load;
mod rotation;
mod stack;
mod timer;
