use gbe_rs::Emulator;
use gbe_rs::cartridge::Cartridge;
use std::io::{self, Write};

const MAX_STEPS: u64 = 25_000_000;

/// Blargg tests communicate their result through the serial port.
///
/// Typical output:
///     Passed
///
/// or:
///     Failed
fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

fn main() -> io::Result<()> {
    let path = match std::env::args().nth(1) {
        Some(path) => path,
        None => {
            eprintln!("usage: blargg_tests <rom>");
            eprintln!(
                "example: cargo run --bin blargg_tests -- \
roms/blargg/cpu_instrs.gb"
            );
            std::process::exit(2);
        }
    };

    println!("========================================");
    println!(" gbe-rs Blargg test runner");
    println!("========================================");
    println!("ROM: {path}");
    println!();

    // ------------------------------------------------------------
    // Cartridge
    // ------------------------------------------------------------

    println!("Loading cartridge...");

    let cartridge = match Cartridge::from_file(&path) {
        Ok(cartridge) => {
            println!("Cartridge loaded successfully.");
            cartridge
        }

        Err(error) => {
            eprintln!("Failed to load cartridge:");
            eprintln!("{error}");
            std::process::exit(1);
        }
    };

    println!();

    // ------------------------------------------------------------
    // Emulator
    // ------------------------------------------------------------

    println!("Creating emulator...");

    let mut emulator = Emulator::new(cartridge);

    println!("Emulator created successfully.");
    println!();
    println!("Running...");
    println!();

    let mut serial_log: Vec<u8> = Vec::new();
    let mut stdout = io::stdout();

    let mut steps: u64 = 0;

    loop {
        emulator.step();
        steps += 1;

        // --------------------------------------------------------
        // Serial
        // --------------------------------------------------------

        if let Some(byte) = emulator.take_serial_output() {
            serial_log.push(byte);

            match byte {
                b'\n' => {
                    write!(stdout, "\n")?;
                }

                0x20..=0x7E => {
                    write!(stdout, "{}", byte as char)?;
                }

                _ => {
                    write!(stdout, "[{byte:02X}]")?;
                }
            }

            stdout.flush()?;

            // ----------------------------------------------------
            // Blargg result
            // ----------------------------------------------------

            if contains(&serial_log, b"Passed") {
                println!();
                println!("========================================");
                println!(" BLARGG TEST PASS");
                println!("========================================");
                println!("steps: {steps}");
                println!("========================================");

                return Ok(());
            }

            if contains(&serial_log, b"Failed") {
                println!();
                println!("========================================");
                println!(" BLARGG TEST FAIL");
                println!("========================================");
                println!("steps: {steps}");
                println!("========================================");

                std::process::exit(1);
            }
        }

        // --------------------------------------------------------
        // Timeout
        // --------------------------------------------------------

        if steps >= MAX_STEPS {
            println!();
            println!("========================================");
            println!(" BLARGG TEST TIMEOUT");
            println!("========================================");
            println!("steps: {steps}");
            println!("serial bytes: {}", serial_log.len());
            println!();

            if serial_log.is_empty() {
                println!("No serial output was produced.");
            } else {
                println!("Serial output:");
                println!("{}", String::from_utf8_lossy(&serial_log));
            }

            println!();
            println!("LCD:");
            println!("  LCDC: ${:02X}", emulator.peek(0xFF40));
            println!("  STAT: ${:02X}", emulator.peek(0xFF41));
            println!("  LY:   ${:02X}", emulator.peek(0xFF44));

            println!("========================================");

            std::process::exit(2);
        }
    }
}
