use gbe_rs::{Emulator, cartridge::Cartridge, frontend};
use log::info;
use std::io;

fn main() -> io::Result<()> {
    env_logger::init();

    let path = std::env::args().nth(1).expect("usage: gbe-rs <rom>");

    let cartridge = Cartridge::from_file(path)?;
    let emulator = Emulator::new(cartridge);

    info!("Main emulator started");

    frontend::run(emulator);

    Ok(())
}
