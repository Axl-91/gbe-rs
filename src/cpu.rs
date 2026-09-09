use crate::{bus::MemoryBus, registers::Registers};

#[allow(dead_code)]

pub struct Cpu {
    registers: Registers,
    bus: MemoryBus,
}
