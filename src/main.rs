use gbe_rs::{Emulator, cartridge::Cartridge};
use log::info;
use std::io;

fn main() -> io::Result<()> {
    env_logger::init();

    let path = std::env::args().nth(1).expect("usage: gbe-rs <rom>");

    let cartridge = Cartridge::from_file(path)?;
    let mut emulator = Emulator::new(cartridge);

    info!("Main emulator started");

    loop {
        emulator.step();
    }
}
