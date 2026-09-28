//! gbmicrotest ROM runner.
//!
//! gbmicrotest ROMs report their result by writing to HRAM:
//! - `0xFF80`: result produced by the test
//! - `0xFF81`: expected result
//! - `0xFF82`: `0x01` if the test passed, `0xFF` if it failed (`0x00` while running)
//!
//! Exit codes:
//! - `0`: the test passed
//! - `1`: the test failed (or the ROM could not be loaded)
//! - `2`: the test timed out (or the arguments were invalid)

use gbe_rs::Emulator;
use gbe_rs::cartridge::Cartridge;
use std::process::ExitCode;

/// Maximum number of emulator steps before the test is considered stuck.
///
/// Most tests finish within a few hundred cycles, but the ones that check
/// behavior after VBLANK need at least one full frame (~70224 cycles), so
/// we leave plenty of headroom.
const MAX_STEPS: u64 = 5_000_000;

/// Address where the test writes the result it produced.
const ADDR_RESULT: u16 = 0xFF80;

/// Address where the test writes the expected result.
const ADDR_EXPECTED: u16 = 0xFF81;

/// Address where the test writes the final verdict.
const ADDR_STATUS: u16 = 0xFF82;

/// Value of `ADDR_STATUS` when the test passed.
const STATUS_PASS: u8 = 0x01;

/// Value of `ADDR_STATUS` when the test failed.
const STATUS_FAIL: u8 = 0xFF;

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
            Outcome::Pass => "GBMICROTEST PASS",
            Outcome::Fail => "GBMICROTEST FAIL",
            Outcome::Timeout => "GBMICROTEST TIMEOUT",
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
    result: u8,
    expected: u8,
    status: u8,
}

/// Captures the three result bytes.
fn snapshot(emulator: &Emulator) -> (u8, u8, u8) {
    (
        emulator.peek(ADDR_RESULT),
        emulator.peek(ADDR_EXPECTED),
        emulator.peek(ADDR_STATUS),
    )
}

/// Reads the ROM path from the command line, or prints usage and exits.
fn parse_rom_path() -> Result<String, ExitCode> {
    std::env::args().nth(1).ok_or_else(|| {
        eprintln!("usage: gbmicrotest <rom>");
        eprintln!("example: cargo run --bin gbmicrotest -- roms/gbmicrotest/000-write_to_x8000.gb");
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

/// Runs the emulator until the ROM reports a verdict or the step limit is hit.
fn run(emulator: &mut Emulator) -> Report {
    for steps in 1..=MAX_STEPS {
        emulator.step();

        let (result, expected, status) = snapshot(emulator);

        let outcome = match status {
            STATUS_PASS => Some(Outcome::Pass),
            STATUS_FAIL => Some(Outcome::Fail),
            _ => None,
        };

        if let Some(outcome) = outcome {
            return Report {
                outcome,
                steps,
                result,
                expected,
                status,
            };
        }
    }

    let (result, expected, status) = snapshot(emulator);

    Report {
        outcome: Outcome::Timeout,
        steps: MAX_STEPS,
        result,
        expected,
        status,
    }
}

/// Prints the final banner with the outcome of the run.
fn print_report(report: &Report) {
    println!();
    println!("{SEPARATOR}");
    println!(" {}", report.outcome.title());
    println!("{SEPARATOR}");
    println!("steps:    {}", report.steps);
    println!("result:   0x{:02X} ({})", report.result, report.result);
    println!("expected: 0x{:02X} ({})", report.expected, report.expected);
    println!("status:   0x{:02X}", report.status);

    if report.outcome == Outcome::Timeout {
        println!("The test never wrote a verdict to 0xFF82.");
    }

    println!("{SEPARATOR}");
}

fn main() -> ExitCode {
    let path = match parse_rom_path() {
        Ok(path) => path,
        Err(code) => return code,
    };

    println!("{SEPARATOR}");
    println!(" gbe-rs gbmicrotest runner");
    println!("{SEPARATOR}");
    println!("ROM: {path}");
    println!();

    let mut emulator = match load_emulator(&path) {
        Ok(emulator) => emulator,
        Err(code) => return code,
    };

    let report = run(&mut emulator);
    print_report(&report);

    report.outcome.exit_code()
}
