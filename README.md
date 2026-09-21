<div align="center">
  <img align="center" width="704" alt="gbe" src="https://github.com/Axl-91/gbe-rs/blob/main/assets/gbe.png" />
</div>

#

A Game Boy DMG emulator written in Rust.

The goal of this project is to build a simple and accurate emulator for the original Game Boy (DMG), starting from the core hardware and gradually adding the components required to run real games.

## Current progress

### CPU
- [x] CPU registers and flags
- [x] Fetch / decode / execute cycle
- [x] Instruction set
  - [x] Arithmetic
  - [x] Load / Store
  - [x] Control flow
  - [x] Stack operations
  - [x] Rotations / shifts
  - [x] Bit operations
  - [x] CPU control instructions
- [x] Instruction timing

### Memory
- [x] Memory bus
- [x] VRAM
- [x] WRAM
- [x] HRAM
- [ ] I/O registers
- [x] Interrupt registers

### Cartridge
- [x] Cartridge ROM
- [x] MBC1
- [ ] MBC2/3
- [x] Cartridge RAM

### Hardware
- [x] Timer
- [x] Interrupts
- [ ] PPU
- [ ] Input

### Frontend
- [ ] Frontend

## Emulator logic

The emulator works by reproducing the main hardware components of the Game Boy and allowing them to interact with each other.

At the center is the CPU, which fetches instructions from memory, decodes them, and executes them. Memory access is handled through a memory bus, which routes reads and writes to the appropriate hardware component.

<img width="1408" alt="gbe_logic" src="https://github.com/Axl-91/gbe-rs/blob/main/assets/diagram.jpg" />

The emulator advances the hardware over time by executing CPU instructions and, eventually, consuming the corresponding number of cycles. This allows components such as the PPU and timer to remain synchronized with the CPU.

## References

- [Pan Docs](https://gbdev.io/pandocs/) — The single most comprehensive technical reference for Game Boy hardware behavior, covering the CPU, memory map, PPU, timers, interrupts, and more.
- [Game Boy: Complete Technical Reference](https://gekkio.fi/files/gb-docs/gbctr.pdf) — Gekkio's in-depth reference (PDF), the primary source for hardware quirks and cycle-accurate behavior.
- [Game Boy Opcode Table](https://gbdev.io/gb-opcodes/optables/) — Instruction reference with opcodes, cycle counts, and flag effects.

# Test Suites

## Blargg

| Test name | Status |
|---|---|
| 11 CPU instructions tests (Isolated) | ✅ Passing |
| CPU instructions tests (Integrated) | ✅ Passing |
| Instructions Timing tests | ✅ Passing |

## Mooneye

| Test name | Status |
|---|---|
| Instructions Timing Tests | ✅ Passing |
| Boot registers test | ✅ Passing |
| Boot DIV test | ✅ Passing |
| DAA instruction tests | ✅ Passing |
| POP Timing tests | ✅ Passing |
| Interrupt tests | ✅ Passing |
| Timing tests | ✅ Passing |
| MBC1 (Up until 4Mb) | ✅ Passing |
| MBC1 (bigger than 4Mb) |❌ Failing|
| MBC2/3| ⏳ Not tested |
| HALT Tests | ⏳ Not tested |

---------------------------------------
<div align="center">
  <img align="center" width="352" alt="gbe" src="https://github.com/Axl-91/gbe-rs/blob/main/assets/Blargg.jpg" />
</div>

