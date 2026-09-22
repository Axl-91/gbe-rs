use gbe_rs::Emulator;
use gbe_rs::cartridge::Cartridge;
use std::io::{self, Write};

/// Bytes sent by a mooneye test ROM on success (Fibonacci 3/5/8/13/21/34).
const MOONEYE_PASS: [u8; 6] = [3, 5, 8, 13, 21, 34];

/// Bytes sent by a mooneye test ROM on failure (0x42 repeated 6 times).
const MOONEYE_FAIL: [u8; 6] = [0x42; 6];

#[derive(Clone, Copy)]
enum Verdict {
    Pass,
    Fail,
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
            return Some(Verdict::Fail);
        }
    }

    // Blargg: only decide once the line is complete, so we don't cut off
    // messages like "Passed all tests" or "Failed #3".
    if log.last() == Some(&b'\n') {
        if contains(log, b"Passed") {
            return Some(Verdict::Pass);
        }
        if contains(log, b"Failed") {
            return Some(Verdict::Fail);
        }
    }

    None
}

fn main() -> io::Result<()> {
    env_logger::init();

    let path = std::env::args().nth(1).expect("usage: gbe-rs <rom>");

    let cartridge = Cartridge::from_file(path)?;
    let mut emulator = Emulator::new(cartridge);

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
                Some(Verdict::Fail) => {
                    println!("\n=> FAIL");
                    std::process::exit(1);
                }
                None => {}
            }
        }
    }
}
