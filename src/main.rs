use gbe_rs::{Emulator, cartridge::Cartridge};
use std::io;

fn main() -> io::Result<()> {
    let path = std::env::args().nth(1).expect("usage: gbe-rs <rom>");

    let cartridge = Cartridge::from_file(path)?;
    let mut emulator = Emulator::new(cartridge);

    loop {
        emulator.step();
    }
}
