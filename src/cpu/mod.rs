pub mod registers;

use crate::memory::MemoryBus;
use registers::Registers;

#[allow(dead_code)]
pub struct Cpu {
    registers: Registers,
    bus: MemoryBus,
}
