pub mod registers;

use crate::memory::bus::MemoryBus;
use registers::Registers;

#[allow(dead_code)]
pub struct Cpu {
    registers: Registers,
    bus: MemoryBus,
}
