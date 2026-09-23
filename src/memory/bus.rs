//! Game Boy memory bus implementation.
//!
//! Routes CPU read and write operations to the appropriate memory component,
//! such as the cartridge, VRAM, and WRAM.

use super::map::*;
use crate::{
    cartridge::Cartridge, dma::Dma, joypad::Joypad, ppu::Ppu, serial::Serial, timer::Timer,
};

/// Value returned when reading from an address that isn't backed by any
/// implemented memory or I/O register.
const UNMAPPED_ADDRESS_VALUE: u8 = 0xFF;

const WRAM_SIZE: usize = 0x2000;
const HRAM_SIZE: usize = 0x7F;

const VBLANK_INTERRUPT: u8 = 0x01;
const STAT_INTERRUPT: u8 = 0x02;

/// Provides access to the Game Boy memory address space.
pub struct MemoryBus {
    cartridge: Cartridge,
    ppu: Ppu,
    wram: [u8; WRAM_SIZE],
    hram: [u8; HRAM_SIZE],
    interrupt_flags: u8,
    interrupt_enable: u8,
    timer: Timer,
    serial: Serial,
    joypad: Joypad,
    dma: Dma,
}

impl MemoryBus {
    /// Creates a new memory bus with the given cartridge and initialized memory.
    pub fn new(cartridge: Cartridge) -> Self {
        Self {
            cartridge,
            ppu: Ppu::new(),
            wram: [0; WRAM_SIZE],
            hram: [0; HRAM_SIZE],
            interrupt_flags: 0xE1,
            interrupt_enable: 0x00,
            timer: Timer::new(),
            serial: Serial::new(),
            joypad: Joypad::new(),
            dma: Dma::new(),
        }
    }

    pub fn is_joypad_active(&self) -> bool {
        self.joypad.is_joypad_active()
    }

    pub fn take_serial_output(&mut self) -> Option<u8> {
        self.serial.take_output()
    }

    fn ppu_tick(&mut self) {
        let ppu_interruptions = self.ppu.tick();
        if ppu_interruptions.vblank {
            self.interrupt_flags |= VBLANK_INTERRUPT;
        }
        if ppu_interruptions.stat {
            self.interrupt_flags |= STAT_INTERRUPT;
        }
    }

    fn dma_tick(&mut self) {
        self.dma.consume_cycle();

        if self.dma.can_transfer_byte() {
            let source_address = self.dma.source_address();
            let index = self.dma.get_index();

            let value = self.read(source_address);
            self.ppu.write(OAM_START + index as u16, value);
        }
        self.dma.tick();
    }

    pub fn tick(&mut self, t_cycles: u8) {
        for _ in 0..t_cycles {
            if self.timer.tick() {
                self.interrupt_flags |= 0x04;
            }
            self.ppu_tick();
            self.dma_tick();
        }
    }

    /// Reads a byte from the given address in the Game Boy memory space.
    pub fn read(&self, address: u16) -> u8 {
        match address {
            CARTRIDGE_ROM_START..=CARTRIDGE_ROM_END => self.cartridge.read(address),

            VRAM_START..=VRAM_END => self.ppu.read(address),

            CARTRIDGE_RAM_START..=CARTRIDGE_RAM_END => self.cartridge.read(address),

            WRAM_START..=WRAM_END => {
                let offset = (address - WRAM_START) as usize;
                self.wram[offset]
            }

            // Echo RAM mirrors WRAM addresses
            ECHO_RAM_START..=ECHO_RAM_END => {
                let offset = (address - ECHO_RAM_START) as usize;
                self.wram[offset]
            }

            OAM_START..=OAM_END => {
                // While DMA is transfering OAM is not accesible
                if self.dma.is_transferring() {
                    0xFF
                } else {
                    self.ppu.read(address)
                }
            }

            UNUSABLE_MEMORY_START..=UNUSABLE_MEMORY_END => 0x00,

            JOYPAD_ADDRESS => self.joypad.read(),

            SERIAL_DATA_ADDRESS => self.serial.read_data(),
            SERIAL_CONTROL_ADDRESS => self.serial.read_control(),

            TIMER_ADDRESS_START..=TIMER_ADDRESS_END => self.timer.read(address),

            INTERRUPT_FLAG_ADDRESS => self.interrupt_flags | 0xE0,

            PPU_REGISTERS_START..=PPU_REGISTERS_END => self.ppu.read(address),

            HRAM_START..=HRAM_END => {
                let offset = (address - HRAM_START) as usize;
                self.hram[offset]
            }

            INTERRUPT_ENABLE_ADDRESS => self.interrupt_enable,

            _ => UNMAPPED_ADDRESS_VALUE,
        }
    }

    /// Writes a byte to the given address in the Game Boy memory space.
    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            CARTRIDGE_ROM_START..=CARTRIDGE_ROM_END => self.cartridge.write(address, value),

            VRAM_START..=VRAM_END => self.ppu.write(address, value),

            CARTRIDGE_RAM_START..=CARTRIDGE_RAM_END => self.cartridge.write(address, value),

            WRAM_START..=WRAM_END => {
                self.wram[(address - WRAM_START) as usize] = value;
            }

            // Echo RAM mirrors WRAM addresses
            ECHO_RAM_START..=ECHO_RAM_END => {
                let offset = (address - ECHO_RAM_START) as usize;
                self.wram[offset] = value;
            }

            OAM_START..=OAM_END => {
                // While DMA is transfering OAM is not accesible
                if !self.dma.is_transferring() {
                    self.ppu.write(address, value)
                }
            }

            UNUSABLE_MEMORY_START..=UNUSABLE_MEMORY_END => {}

            JOYPAD_ADDRESS => self.joypad.write(value),

            SERIAL_DATA_ADDRESS => self.serial.write_data(value),
            SERIAL_CONTROL_ADDRESS => self.serial.write_control(value),

            TIMER_ADDRESS_START..=TIMER_ADDRESS_END => self.timer.write(address, value),

            INTERRUPT_FLAG_ADDRESS => self.interrupt_flags = value,

            PPU_REGISTERS_START..=PPU_REGISTERS_END => match address {
                DMA_ADDRESS => self.dma.start(value),
                _ => self.ppu.write(address, value),
            },

            HRAM_START..=HRAM_END => {
                self.hram[(address - HRAM_START) as usize] = value;
            }

            INTERRUPT_ENABLE_ADDRESS => self.interrupt_enable = value,

            _ => {}
        }
    }
}
