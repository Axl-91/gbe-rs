//! Game Boy memory system.
//!
//! Provides the memory bus and address map used by the CPU to access hardware.

pub mod bus;
pub mod map;

pub use bus::MemoryBus;
