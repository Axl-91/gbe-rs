use gbe_rs::Emulator;
use gbe_rs::cartridge::Cartridge;
use std::io::{self, Write};

/// Mooneye success sequence:
///
/// 03 05 08 0D 15 22
///
/// This is:
/// 3, 5, 8, 13, 21, 34
const MOONEYE_PASS: [u8; 6] = [3, 5, 8, 13, 21, 34];

/// Mooneye failure sequence:
///
/// 42 42 42 42 42 42
const MOONEYE_FAIL: [u8; 6] = [0x42, 0x42, 0x42, 0x42, 0x42, 0x42];

/// HRAM values written by Mooneye's failure routine.
const HRAM_FAIL_ROUND: u16 = 0xFF98;
const HRAM_FAIL_EXPECT: u16 = 0xFF99;
const HRAM_FAIL_ACTUAL: u16 = 0xFF9A;
const HRAM_FAIL_STR_L: u16 = 0xFF9B;
const HRAM_FAIL_STR_H: u16 = 0xFF9C;

const DEBUG_INTERVAL: u64 = 10_000;
const MAX_STEPS: u64 = 5_000_000;

fn read_rom_string(emulator: &Emulator, mut addr: u16, max_len: usize) -> String {
    let mut result = String::new();

    for _ in 0..max_len {
        let byte = emulator.peek(addr);

        if byte == 0 {
            break;
        }

        result.push(byte as char);
        addr = addr.wrapping_add(1);
    }

    result
}

fn decode_fail_round(round: u8) -> (u8, u8, u16) {
    const OFFSETS_M: [u16; 8] = [0, 17, 60, 110, 130, 174, 224, 244];

    let pass = round / 8;
    let read_idx = round % 8;

    let t_cycles = (OFFSETS_M[read_idx as usize] + pass as u16) * 4;

    (pass, read_idx, t_cycles)
}

fn print_failure_details(emulator: &Emulator) {
    let round = emulator.peek(HRAM_FAIL_ROUND);
    let expected = emulator.peek(HRAM_FAIL_EXPECT);
    let actual = emulator.peek(HRAM_FAIL_ACTUAL);

    let str_l = emulator.peek(HRAM_FAIL_STR_L);
    let str_h = emulator.peek(HRAM_FAIL_STR_H);

    println!();
    println!("Mooneye failure details:");

    if round > 23 {
        println!("  No valid failure round in FF98: ${round:02X}");
        return;
    }

    let string_address = u16::from_le_bytes([str_l, str_h]);

    println!("  round:    {round}");
    println!("  expected: ${expected:02X}");
    println!("  actual:   ${actual:02X}");
    println!("  string:   ${string_address:04X}");

    if !(0x0100..=0x7FFF).contains(&string_address) {
        println!("  name:     <invalid ROM address>");
        return;
    }

    let name = read_rom_string(emulator, string_address, 32);

    if name.is_empty() {
        println!("  name:     <empty>");
        return;
    }

    let (pass, read_index, t_cycles) = decode_fail_round(round);

    println!("  name:     {name}");
    println!("  pass:     {}", pass + 1);
    println!("  read:     {}", read_index + 1);
    println!("  cycle:    ~T{t_cycles}");
}

fn main() -> io::Result<()> {
    let path = match std::env::args().nth(1) {
        Some(path) => path,
        None => {
            eprintln!("usage: mooneye_tests <rom>");
            eprintln!(
                "example: cargo run --bin mooneye_tests -- \
roms/mooneye/acceptance/..."
            );
            std::process::exit(2);
        }
    };

    println!("========================================");
    println!(" gbe-rs Mooneye test runner");
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

            // Display printable bytes normally and control bytes
            // as hexadecimal.
            match byte {
                b'\n' => {
                    writeln!(stdout)?;
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
            // Check last 6 bytes.
            // ----------------------------------------------------

            if serial_log.len() >= 6 {
                let tail = &serial_log[serial_log.len() - 6..];

                if tail == MOONEYE_PASS {
                    println!();
                    println!("========================================");
                    println!(" MOONEYE TEST PASS");
                    println!("========================================");
                    println!("steps: {steps}");
                    println!("========================================");

                    return Ok(());
                }

                if tail == MOONEYE_FAIL {
                    println!();
                    println!("========================================");
                    println!(" MOONEYE TEST FAIL");
                    println!("========================================");
                    println!("steps: {steps}");
                    println!("========================================");

                    print_failure_details(&emulator);

                    std::process::exit(1);
                }
            }
        }

        // --------------------------------------------------------
        // Progress
        // --------------------------------------------------------

        if steps.is_multiple_of(DEBUG_INTERVAL) {
            let lcdc = emulator.peek(0xFF40);
            let stat = emulator.peek(0xFF41);
            let ly = emulator.peek(0xFF44);

            println!();
            println!(
                "[debug] steps={steps} serial_bytes={} \
LCDC=${lcdc:02X} STAT=${stat:02X} LY=${ly:02X}",
                serial_log.len()
            );
        }

        // --------------------------------------------------------
        // Timeout
        // --------------------------------------------------------

        if steps >= MAX_STEPS {
            println!();
            println!("========================================");
            println!(" MOONEYE TEST TIMEOUT");
            println!("========================================");
            println!("steps: {steps}");
            println!("serial bytes: {}", serial_log.len());
            println!();

            if serial_log.is_empty() {
                println!("No serial output was produced.");
            } else {
                print!("Serial output: ");

                for byte in &serial_log {
                    match byte {
                        0x20..=0x7E => {
                            print!("{}", *byte as char);
                        }

                        _ => {
                            print!("[{byte:02X}]");
                        }
                    }
                }

                println!();
            }

            println!();
            println!("LCD:");
            println!("  LCDC: ${:02X}", emulator.peek(0xFF40));
            println!("  STAT: ${:02X}", emulator.peek(0xFF41));
            println!("  LY:   ${:02X}", emulator.peek(0xFF44));
            println!("  LYC:  ${:02X}", emulator.peek(0xFF45));

            println!();
            println!("Interrupts:");
            println!("  IF:   ${:02X}", emulator.peek(0xFF0F));
            println!("  IE:   ${:02X}", emulator.peek(0xFFFF));

            println!("========================================");

            std::process::exit(2);
        }
    }
}
