use kaniboy::{Emulator, bootrom::BootRom, cartridge::Cartridge, frontend};
use log::info;
use std::{io, path::PathBuf, process};

const USAGE: &str = "usage: kaniboy [--boot <boot_rom>] <rom>";

struct Args {
    boot_rom: Option<PathBuf>,
    rom: PathBuf,
}

fn parse_args() -> Result<Args, String> {
    let mut args = std::env::args().skip(1);
    let mut boot_rom = None;
    let mut rom = None;

    while let Some(arg) = args.next() {
        if arg == "--boot" {
            let path = args.next().ok_or("--boot requires a path")?;
            boot_rom = Some(PathBuf::from(path));
        } else if let Some(path) = arg.strip_prefix("--boot=") {
            boot_rom = Some(PathBuf::from(path));
        } else if arg.starts_with("--") {
            return Err(format!("unknown option {arg}"));
        } else if rom.replace(PathBuf::from(arg)).is_some() {
            return Err("only one ROM can be given".to_string());
        }
    }

    let rom = rom.ok_or("missing ROM path")?;

    Ok(Args { boot_rom, rom })
}

fn main() -> io::Result<()> {
    env_logger::init();

    let args = parse_args().unwrap_or_else(|error| {
        eprintln!("error: {error}\n{USAGE}");
        process::exit(2);
    });

    let cartridge = Cartridge::from_file(&args.rom)?;

    let emulator = match args.boot_rom {
        Some(path) => {
            let boot_rom = BootRom::from_file(&path).map_err(|error| {
                io::Error::new(
                    error.kind(),
                    format!("cannot load boot ROM {}: {error}", path.display()),
                )
            })?;
            info!("Running boot ROM {}", path.display());
            Emulator::with_boot_rom(cartridge, boot_rom)
        }
        None => Emulator::new(cartridge),
    };

    info!("Main emulator started");

    frontend::run(emulator);

    Ok(())
}
