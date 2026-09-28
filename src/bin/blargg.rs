//! Blargg test ROM runner.
//!
//! Blargg's test ROMs report their result through the serial port by
//! printing a human-readable message that contains either `Passed` or
//! `Failed`.
//!
//! Exit codes:
//! - `0`: the test passed
//! - `1`: the test failed (or the ROM could not be loaded)
//! - `2`: the test timed out (or the arguments were invalid)

use gbe_rs::Emulator;
use gbe_rs::cartridge::Cartridge;
use std::io::{self, Write};
use std::process::ExitCode;

/// Maximum number of emulator steps before the test is considered stuck.
const MAX_STEPS: u64 = 25_000_000;

/// Serial output that indicates a successful run.
const PASS_MARKER: &[u8] = b"Passed";

/// Serial output that indicates a failed run.
const FAIL_MARKER: &[u8] = b"Failed";

/// Width of the banner separator lines.
const SEPARATOR: &str = "========================================";

/// Final result of a test run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outcome {
    Pass,
    Fail,
    Timeout,
}

impl Outcome {
    fn title(self) -> &'static str {
        match self {
            Outcome::Pass => "BLARGG TEST PASS",
            Outcome::Fail => "BLARGG TEST FAIL",
            Outcome::Timeout => "BLARGG TEST TIMEOUT",
        }
    }

    fn exit_code(self) -> ExitCode {
        match self {
            Outcome::Pass => ExitCode::SUCCESS,
            Outcome::Fail => ExitCode::from(1),
            Outcome::Timeout => ExitCode::from(2),
        }
    }
}

/// Everything collected while running a test ROM.
struct Report {
    outcome: Outcome,
    steps: u64,
    serial_log: Vec<u8>,
}

/// Returns `true` if `needle` appears anywhere inside `haystack`.
fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

/// Formats a serial byte for display: newlines and printable ASCII are
/// shown as-is, anything else is shown as a hexadecimal escape.
fn format_serial_byte(byte: u8) -> String {
    match byte {
        b'\n' => "\n".to_string(),
        0x20..=0x7E => (byte as char).to_string(),
        _ => format!("[{byte:02X}]"),
    }
}

/// Reads the ROM path from the command line, or prints usage and exits.
fn parse_rom_path() -> Result<String, ExitCode> {
    std::env::args().nth(1).ok_or_else(|| {
        eprintln!("usage: blargg_tests <rom>");
        eprintln!("example: cargo run --bin blargg_tests -- roms/blargg/cpu_instrs.gb");
        ExitCode::from(2)
    })
}

/// Loads the cartridge at `path` and builds an emulator around it.
fn load_emulator(path: &str) -> Result<Emulator, ExitCode> {
    match Cartridge::from_file(path) {
        Ok(cartridge) => Ok(Emulator::new(cartridge)),
        Err(error) => {
            eprintln!("Failed to load cartridge: {error}");
            Err(ExitCode::from(1))
        }
    }
}

/// Runs the emulator until the ROM reports a result or the step limit is hit.
///
/// Serial output is echoed to stdout as it arrives.
fn run(emulator: &mut Emulator) -> io::Result<Report> {
    let mut stdout = io::stdout();
    let mut serial_log: Vec<u8> = Vec::new();

    for steps in 1..=MAX_STEPS {
        emulator.step();

        let Some(byte) = emulator.take_serial_output() else {
            continue;
        };

        serial_log.push(byte);
        write!(stdout, "{}", format_serial_byte(byte))?;
        stdout.flush()?;

        let outcome = if contains(&serial_log, PASS_MARKER) {
            Some(Outcome::Pass)
        } else if contains(&serial_log, FAIL_MARKER) {
            Some(Outcome::Fail)
        } else {
            None
        };

        if let Some(outcome) = outcome {
            return Ok(Report {
                outcome,
                steps,
                serial_log,
            });
        }
    }

    Ok(Report {
        outcome: Outcome::Timeout,
        steps: MAX_STEPS,
        serial_log,
    })
}

/// Prints the final banner with the outcome of the run.
fn print_report(report: &Report) {
    println!();
    println!("{SEPARATOR}");
    println!(" {}", report.outcome.title());
    println!("{SEPARATOR}");
    println!("steps: {}", report.steps);

    if report.outcome == Outcome::Timeout {
        println!("serial bytes: {}", report.serial_log.len());

        if report.serial_log.is_empty() {
            println!("No serial output was produced.");
        }
    }

    println!("{SEPARATOR}");
}

fn main() -> io::Result<ExitCode> {
    let path = match parse_rom_path() {
        Ok(path) => path,
        Err(code) => return Ok(code),
    };

    println!("{SEPARATOR}");
    println!(" gbe-rs Blargg test runner");
    println!("{SEPARATOR}");
    println!("ROM: {path}");
    println!();

    let mut emulator = match load_emulator(&path) {
        Ok(emulator) => emulator,
        Err(code) => return Ok(code),
    };

    let report = run(&mut emulator)?;
    print_report(&report);

    Ok(report.outcome.exit_code())
}
