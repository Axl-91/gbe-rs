use gbe_rs::{cartridge::Cartridge, cpu::Cpu, memory::MemoryBus};
use std::io::{self, Write};

fn main() -> io::Result<()> {
    let path = std::env::args().nth(1).expect("usage: gbe-rs <rom>");

    let cartridge = Cartridge::from_file(path)?;
    let bus = MemoryBus::new(cartridge);
    let mut cpu = Cpu::new(bus);

    loop {
        cpu.step();

        if let Some(byte) = cpu.take_serial_output() {
            print!("{}", byte as char);
            io::stdout().flush()?;
        }
    }
}
