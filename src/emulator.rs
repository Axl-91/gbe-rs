use crate::{bootrom::BootRom, cartridge::Cartridge, cpu::Cpu, joypad::Button, memory::MemoryBus};
use std::io;

pub struct Emulator {
    cpu: Cpu,
}

impl Emulator {
    /// Starts at the cartridge entry point ($0100), skipping the boot ROM.
    pub fn new(cartridge: Cartridge) -> Self {
        Self::from_bus(MemoryBus::new(cartridge))
    }

    /// Runs `boot_rom` first, as on real hardware.
    pub fn with_boot_rom(cartridge: Cartridge, boot_rom: BootRom) -> Self {
        Self::from_bus(MemoryBus::with_boot_rom(cartridge, boot_rom))
    }

    fn from_bus(bus: MemoryBus) -> Self {
        Self { cpu: Cpu::new(bus) }
    }

    pub fn step(&mut self) -> u8 {
        self.cpu.step()
    }

    pub fn take_serial_output(&mut self) -> Option<u8> {
        self.cpu.take_serial_output()
    }

    pub fn peek(&self, address: u16) -> u8 {
        self.cpu.peek(address)
    }

    pub fn press_button(&mut self, button: Button) {
        self.cpu.joypad_press(button);
    }

    pub fn release_button(&mut self, button: Button) {
        self.cpu.joypad_release(button);
    }

    pub fn get_framebuffer(&self) -> &[u8] {
        self.cpu.get_framebuffer()
    }

    pub fn take_audio_samples(&mut self) -> Vec<(f32, f32)> {
        self.cpu.take_audio_samples()
    }

    pub fn save(&self) -> io::Result<()> {
        self.cpu.save()
    }
}
