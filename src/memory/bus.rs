//! Game Boy memory bus implementation.
//!
//! Routes CPU read and write operations to the appropriate memory component,
//! such as the cartridge, VRAM, and WRAM.

use log::info;

use super::map::*;
use crate::{
    apu::Apu,
    bootrom::BootRom,
    cartridge::Cartridge,
    dma::Dma,
    joypad::{Button, Joypad},
    ppu::{Ppu, oam_corrupt::OamCorruption},
    serial::Serial,
    timer::Timer,
};
use std::io;

/// Value returned when reading from an address that isn't backed by any
/// implemented memory or I/O register.
const UNMAPPED_ADDRESS_VALUE: u8 = 0xFF;

const WRAM_SIZE: usize = 0x2000;
const HRAM_SIZE: usize = 0x7F;

// bit to set interruptions
const VBLANK_INTERRUPT: u8 = 0b0000_0001;
const STAT_INTERRUPT: u8 = 0b0000_0010;
const JOYPAD_INTERRUPT: u8 = 0b0001_0000;

/// Provides access to the Game Boy memory address space.
pub struct MemoryBus {
    boot_rom: Option<BootRom>,
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
    apu: Apu,
}

impl MemoryBus {
    /// Boot ROM skipped: components start in their post-boot state.
    pub fn new(cartridge: Cartridge) -> Self {
        Self::build(cartridge, None)
    }

    /// Boot ROM mapped at $0000: components start in their power-on state.
    pub fn with_boot_rom(cartridge: Cartridge, boot_rom: BootRom) -> Self {
        Self::build(cartridge, Some(boot_rom))
    }

    fn build(cartridge: Cartridge, boot_rom: Option<BootRom>) -> Self {
        let booting = boot_rom.is_some();

        Self {
            cartridge,
            boot_rom,
            ppu: if booting {
                Ppu::power_on()
            } else {
                Ppu::post_boot()
            },
            wram: [0; WRAM_SIZE],
            hram: [0; HRAM_SIZE],
            interrupt_flags: if booting { 0x00 } else { 0xE1 },
            interrupt_enable: 0x00,
            timer: if booting {
                Timer::power_on()
            } else {
                Timer::post_boot()
            },
            // serial, joypad, dma: sin cambios por ahora
            serial: Serial::new(),
            joypad: Joypad::new(),
            dma: Dma::new(),
            apu: if booting {
                Apu::power_on()
            } else {
                Apu::post_boot()
            },
        }
    }

    pub fn is_boot_rom_enabled(&self) -> bool {
        self.boot_rom.as_ref().is_some_and(BootRom::is_enabled)
    }

    pub fn get_framebuffer(&self) -> &[u8] {
        self.ppu.get_framebuffer()
    }

    pub fn take_audio_samples(&mut self) -> Vec<(f32, f32)> {
        self.apu.take_samples()
    }

    pub fn save(&self) -> io::Result<()> {
        self.cartridge.save()
    }

    pub fn is_joypad_active(&self) -> bool {
        self.joypad.is_joypad_active()
    }

    pub fn joypad_press(&mut self, button: Button) {
        let has_interruption = self.joypad.press(button);

        if has_interruption {
            self.interrupt_flags |= JOYPAD_INTERRUPT
        }
    }

    pub fn joypad_release(&mut self, button: Button) {
        self.joypad.release(button);
    }

    pub fn is_oam_search(&self) -> bool {
        self.ppu.is_oam_search()
    }

    pub(crate) fn oam_corrupt(&mut self, corruption: OamCorruption) {
        self.ppu.oam_corrupt(corruption);
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
            self.ppu.dma_write_oam(OAM_START + index as u16, value);
        }
        self.dma.tick();
    }

    pub fn tick(&mut self, t_cycles: u8) {
        for _ in 0..t_cycles {
            if self.timer.tick() {
                self.interrupt_flags |= 0x04;
            }
            self.ppu_tick();
            self.apu.tick();
            self.dma_tick();
        }
    }

    /// Reads a byte from the given address in the Game Boy memory space.
    pub fn read(&self, address: u16) -> u8 {
        if address <= BOOT_ROM_END
            && let Some(boot_rom) = &self.boot_rom
            && boot_rom.is_enabled()
        {
            return boot_rom.read(address);
        }

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

            APU_REGISTERS_START..=APU_REGISTERS_END => self.apu.read(address),

            WAVE_RAM_START..=WAVE_RAM_END => self.apu.read(address),

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

            JOYPAD_ADDRESS => {
                let has_interruption = self.joypad.write(value);
                if has_interruption {
                    self.interrupt_flags |= JOYPAD_INTERRUPT;
                }
            }

            SERIAL_DATA_ADDRESS => self.serial.write_data(value),
            SERIAL_CONTROL_ADDRESS => self.serial.write_control(value),

            TIMER_ADDRESS_START..=TIMER_ADDRESS_END => self.timer.write(address, value),

            INTERRUPT_FLAG_ADDRESS => self.interrupt_flags = value,

            APU_REGISTERS_START..=APU_REGISTERS_END => self.apu.write(address, value),

            WAVE_RAM_START..=WAVE_RAM_END => self.apu.write(address, value),

            PPU_REGISTERS_START..=PPU_REGISTERS_END => match address {
                DMA_ADDRESS => self.dma.start(value),
                _ => {
                    self.ppu.write(address, value);

                    // Some writes on PPU can result in an interruption
                    if self.ppu.has_interruptions() {
                        self.interrupt_flags |= STAT_INTERRUPT;
                    }
                }
            },

            BOOT_ROM_DISABLE_ADDRESS => {
                if value != 0 && self.is_boot_rom_enabled() {
                    if let Some(boot_rom) = &mut self.boot_rom {
                        boot_rom.disable();
                    }
                    info!("Boot ROM disabled, cartridge mapped at $0000");
                }
            }

            HRAM_START..=HRAM_END => {
                self.hram[(address - HRAM_START) as usize] = value;
            }

            INTERRUPT_ENABLE_ADDRESS => self.interrupt_enable = value,

            _ => {}
        }
    }
}
