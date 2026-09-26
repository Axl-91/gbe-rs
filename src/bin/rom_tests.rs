use gbe_rs::Emulator;
use gbe_rs::cartridge::Cartridge;
use log::info;
use std::io::{self, Write};

/// Bytes sent by a mooneye test ROM on success (Fibonacci 3/5/8/13/21/34).
const MOONEYE_PASS: [u8; 6] = [3, 5, 8, 13, 21, 34];

/// Bytes sent by a mooneye test ROM on failure (0x42 repeated 6 times).
const MOONEYE_FAIL: [u8; 6] = [0x42; 6];

/// HRAM addresses where mooneye's `verify_fail` routine stores failure
/// details before the ROM ever tries to draw anything on screen.
const HRAM_FAIL_ROUND: u16 = 0xFF98;
const HRAM_FAIL_EXPECT: u16 = 0xFF99;
const HRAM_FAIL_ACTUAL: u16 = 0xFF9A;
const HRAM_FAIL_STR_L: u16 = 0xFF9B;
const HRAM_FAIL_STR_H: u16 = 0xFF9C;

#[derive(Clone, Copy)]
enum TestKind {
    Mooneye,
    Blargg,
}

#[derive(Clone, Copy)]
enum Verdict {
    Pass,
    Fail(TestKind),
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}

/// Looks at the serial log and decides whether the test has finished.
/// Understands both mooneye (magic byte sequences) and Blargg (ASCII text).
fn check_verdict(log: &[u8]) -> Option<Verdict> {
    // Mooneye
    if log.len() >= 6 {
        let tail = &log[log.len() - 6..];

        if tail == MOONEYE_PASS {
            return Some(Verdict::Pass);
        }

        if tail == MOONEYE_FAIL {
            return Some(Verdict::Fail(TestKind::Mooneye));
        }
    }

    // Blargg
    if log.last() == Some(&b'\n') {
        if contains(log, b"Passed") {
            return Some(Verdict::Pass);
        }

        if contains(log, b"Failed") {
            return Some(Verdict::Fail(TestKind::Blargg));
        }
    }

    None
}

/// Reads a null-terminated ASCII string embedded in ROM (used by
/// mooneye's test-name labels, e.g. "LY", "STAT LYC=0").
fn read_rom_string(emulator: &Emulator, mut addr: u16, max_len: usize) -> String {
    let mut s = String::new();

    for _ in 0..max_len {
        let byte = emulator.peek(addr);

        if byte == 0 {
            break;
        }

        s.push(byte as char);
        addr = addr.wrapping_add(1);
    }

    s
}

/// Turns mooneye's `fail_round` index (0-23) into a human-readable
/// (pass, read-index, approximate T-cycle offset) triple, based on the
/// cycle_counts table shared across the `-GS` PPU tests:
/// offsets 0,17,60,110,130,174,224,244 (M-cycles), one column per pass
/// (pass N adds N extra M-cycles of delay before its reads).
fn decode_fail_round(round: u8) -> (u8, u8, u16) {
    const OFFSETS_M: [u16; 8] = [0, 17, 60, 110, 130, 174, 224, 244];

    let pass = round / 8;
    let read_idx = round % 8;
    let t_cycles = (OFFSETS_M[read_idx as usize] + pass as u16) * 4;

    (pass, read_idx, t_cycles)
}

fn print_mooneye_failure_details(emulator: &Emulator) {
    let round = emulator.peek(HRAM_FAIL_ROUND);
    let expected = emulator.peek(HRAM_FAIL_EXPECT);
    let actual = emulator.peek(HRAM_FAIL_ACTUAL);
    let str_l = emulator.peek(HRAM_FAIL_STR_L);
    let str_h = emulator.peek(HRAM_FAIL_STR_H);

    // Mooneye's failure routine stores a round in the range 0..=23.
    // If this value is not valid, this ROM probably does not provide
    // the expected failure information in HRAM.
    if round > 23 {
        eprintln!("No additional Mooneye failure details available.");
        return;
    }

    let str_addr = u16::from_le_bytes([str_l, str_h]);

    // The test name must point into cartridge ROM.
    if !(0x0100..=0x7FFF).contains(&str_addr) {
        eprintln!("No additional Mooneye failure details available.");
        return;
    }

    let name = read_rom_string(emulator, str_addr, 32);

    if name.is_empty() {
        eprintln!("No additional Mooneye failure details available.");
        return;
    }

    let (pass, read_idx, t_cycles) = decode_fail_round(round);

    eprintln!("Test failed: {name}");
    eprintln!(
        "  round:    {round} (pass {}, read #{})",
        pass + 1,
        read_idx + 1
    );
    eprintln!("  cycle:    ~T{t_cycles}");
    eprintln!("  expected: ${expected:02X}");
    eprintln!("  actual:   ${actual:02X}");
}

fn main() -> io::Result<()> {
    env_logger::init();

    let path = std::env::args().nth(1).expect("usage: gbe-rs <rom>");

    let cartridge = Cartridge::from_file(path)?;
    let mut emulator = Emulator::new(cartridge);

    info!("Rom tests emulator started");

    let mut serial_log: Vec<u8> = Vec::new();
    let mut stdout = io::stdout();

    loop {
        emulator.step();

        if let Some(byte) = emulator.take_serial_output() {
            serial_log.push(byte);

            // Blargg sends readable text; mooneye sends control bytes,
            // so show those as hex to keep the output legible.
            match byte {
                b'\n' | 0x20..=0x7E => write!(stdout, "{}", byte as char)?,
                _ => write!(stdout, "[{byte:02X}]")?,
            }

            stdout.flush()?;

            match check_verdict(&serial_log) {
                Some(Verdict::Pass) => {
                    println!("\n=> PASS");
                    return Ok(());
                }

                Some(Verdict::Fail(TestKind::Mooneye)) => {
                    println!("\n=> FAIL");
                    print_mooneye_failure_details(&emulator);
                    std::process::exit(1);
                }

                Some(Verdict::Fail(TestKind::Blargg)) => {
                    println!("\n=> FAIL");
                    std::process::exit(1);
                }

                None => {}
            }
        }
    }
}
