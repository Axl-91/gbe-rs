use crate::{cartridge::Cartridge, cpu::Cpu, joypad::Button, memory::MemoryBus};

pub struct Emulator {
    cpu: Cpu,
}

impl Emulator {
    pub fn new(cartridge: Cartridge) -> Self {
        let bus = MemoryBus::new(cartridge);
        let cpu = Cpu::new(bus);

        Self { cpu }
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
}
