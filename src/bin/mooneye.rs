//! Mooneye test ROM runner.
//!
//! Mooneye's test ROMs report their result through the serial port using
//! a fixed six-byte sequence:
//!
//! - Pass: the Fibonacci numbers `3, 5, 8, 13, 21, 34`
//! - Fail: `0x42` repeated six times
//!
//! On failure, the ROM also leaves diagnostic information in HRAM, which
//! this runner decodes and prints.
//!
//! Exit codes:
//! - `0`: the test passed
//! - `1`: the test failed (or the ROM could not be loaded)
//! - `2`: the test timed out (or the arguments were invalid)

use gbe_rs::Emulator;
use gbe_rs::cartridge::Cartridge;
use std::io::{self, Write};
use std::ops::RangeInclusive;
use std::process::ExitCode;

/// Maximum number of emulator steps before the test is considered stuck.
const MAX_STEPS: u64 = 5_000_000;

/// Width of the banner separator lines.
const SEPARATOR: &str = "========================================";

/// Serial sequence written by Mooneye on success: 3, 5, 8, 13, 21, 34.
const MOONEYE_PASS: [u8; 6] = [3, 5, 8, 13, 21, 34];

/// Serial sequence written by Mooneye on failure.
const MOONEYE_FAIL: [u8; 6] = [0x42; 6];

// HRAM locations written by Mooneye's failure routine.
const HRAM_FAIL_ROUND: u16 = 0xFF98;
const HRAM_FAIL_EXPECT: u16 = 0xFF99;
const HRAM_FAIL_ACTUAL: u16 = 0xFF9A;
const HRAM_FAIL_STR_L: u16 = 0xFF9B;
const HRAM_FAIL_STR_H: u16 = 0xFF9C;

/// Highest valid failure round (3 passes x 8 reads = rounds 0..=23).
const MAX_FAIL_ROUND: u8 = 23;

/// Number of reads performed per pass by the failing test.
const READS_PER_PASS: u8 = 8;

/// Machine-cycle offsets of each read within a pass.
const READ_OFFSETS_M: [u16; 8] = [0, 17, 60, 110, 130, 174, 224, 244];

/// Addresses where a failure description string can legitimately live.
const ROM_STRING_RANGE: RangeInclusive<u16> = 0x0100..=0x7FFF;

/// Maximum length of a failure description string.
const MAX_NAME_LEN: usize = 32;

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
            Outcome::Pass => "MOONEYE TEST PASS",
            Outcome::Fail => "MOONEYE TEST FAIL",
            Outcome::Timeout => "MOONEYE TEST TIMEOUT",
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

/// Formats a serial byte for display: newlines and printable ASCII are
/// shown as-is, anything else is shown as a hexadecimal escape.
fn format_serial_byte(byte: u8) -> String {
    match byte {
        b'\n' => "\n".to_string(),
        0x20..=0x7E => (byte as char).to_string(),
        _ => format!("[{byte:02X}]"),
    }
}

/// Reads a NUL-terminated string starting at `addr`, up to `max_len` bytes.
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

/// Decodes a failure round into `(pass, read_index, approximate_t_cycles)`.
///
/// Both `pass` and `read_index` are zero-based.
fn decode_fail_round(round: u8) -> (u8, u8, u16) {
    let pass = round / READS_PER_PASS;
    let read_index = round % READS_PER_PASS;

    let m_cycles = READ_OFFSETS_M[read_index as usize] + u16::from(pass);
    let t_cycles = m_cycles * 4;

    (pass, read_index, t_cycles)
}

/// Prints the diagnostic information left in HRAM by a failing test.
fn print_failure_details(emulator: &Emulator) {
    let round = emulator.peek(HRAM_FAIL_ROUND);
    let expected = emulator.peek(HRAM_FAIL_EXPECT);
    let actual = emulator.peek(HRAM_FAIL_ACTUAL);
    let string_address = u16::from_le_bytes([
        emulator.peek(HRAM_FAIL_STR_L),
        emulator.peek(HRAM_FAIL_STR_H),
    ]);

    println!();
    println!("Mooneye failure details:");

    if round > MAX_FAIL_ROUND {
        println!("  No valid failure round in FF98: ${round:02X}");
        return;
    }

    println!("  round:    {round}");
    println!("  expected: ${expected:02X}");
    println!("  actual:   ${actual:02X}");
    println!("  string:   ${string_address:04X}");

    if !ROM_STRING_RANGE.contains(&string_address) {
        println!("  name:     <invalid ROM address>");
        return;
    }

    let name = read_rom_string(emulator, string_address, MAX_NAME_LEN);

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

/// Reads the ROM path from the command line, or prints usage and exits.
fn parse_rom_path() -> Result<String, ExitCode> {
    std::env::args().nth(1).ok_or_else(|| {
        eprintln!("usage: mooneye_tests <rom>");
        eprintln!("example: cargo run --bin mooneye_tests -- roms/mooneye/acceptance/...");
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

        let outcome = if serial_log.ends_with(&MOONEYE_PASS) {
            Some(Outcome::Pass)
        } else if serial_log.ends_with(&MOONEYE_FAIL) {
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
        } else {
            let text: String = report
                .serial_log
                .iter()
                .map(|&byte| format_serial_byte(byte))
                .collect();

            println!("Serial output: {text}");
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
    println!(" gbe-rs Mooneye test runner");
    println!("{SEPARATOR}");
    println!("ROM: {path}");
    println!();

    let mut emulator = match load_emulator(&path) {
        Ok(emulator) => emulator,
        Err(code) => return Ok(code),
    };

    let report = run(&mut emulator)?;
    print_report(&report);

    if report.outcome == Outcome::Fail {
        print_failure_details(&emulator);
    }

    Ok(report.outcome.exit_code())
}
