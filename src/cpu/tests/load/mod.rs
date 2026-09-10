use rand::RngExt;

use super::super::*;

use crate::cartridge::Cartridge;
use crate::memory::MemoryBus;
use crate::memory::map::ROM_BANK_SIZE;

fn create_cpu(opcode: u8, value: u8) -> Cpu {
    let mut rom = vec![0; ROM_BANK_SIZE * 2];

    rom[GAME_ENTRY_POINT as usize] = opcode;
    rom[GAME_ENTRY_POINT as usize + 1] = value;

    let cartridge = Cartridge::new(rom, 0);
    let bus = MemoryBus::new(cartridge);

    Cpu::new(bus)
}

mod hl;
mod immediate;
mod register;
