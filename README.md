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
- [ ] MBC
  - [x] MBC1
  - [ ] MBC2
  - [ ] MBC3
- [x] Cartridge RAM

### Hardware
- [x] Timer
- [x] Interrupts
- [ ] PPU
  - [x] Memory
  - [x] Registers
  - [x] Timing
  - [ ] Rendering
- [ ] Input

### Frontend
- [ ] Frontend

## Emulator architecture

The emulator reproduces the main hardware components of the Game Boy and allows them to interact through a shared memory bus and a cycle-based execution model.

At the center is the CPU, which fetches instructions from memory, decodes them, and executes them. Memory access is handled through the memory bus, which routes reads and writes to the appropriate hardware component, such as the cartridge, RAM, PPU, timer, serial interface, or joypad.

<img width="1408" alt="gbe_logic" src="https://github.com/Axl-91/gbe-rs/blob/main/assets/diagram.jpg" />

The emulator advances the hardware over time using the CPU's clock cycles. Each executed instruction consumes a specific number of cycles, and the corresponding hardware components are advanced accordingly. This keeps components such as the PPU, timer, DMA controller, and serial interface synchronized with the CPU.

Hardware components are responsible for their own internal state and timing, while the emulator coordinates their execution and the communication between them. This approach keeps the different parts of the emulator modular while allowing them to behave as a single system.

## References

- [Pan Docs](https://gbdev.io/pandocs/) — The single most comprehensive technical reference for Game Boy hardware behavior, covering the CPU, memory map, PPU, timers, interrupts, and more.
- [Game Boy: Complete Technical Reference](https://gekkio.fi/files/gb-docs/gbctr.pdf) — Gekkio's in-depth reference (PDF), the primary source for hardware quirks and cycle-accurate behavior.
- [Game Boy Opcode Table](https://gbdev.io/gb-opcodes/optables/) — Instruction reference with opcodes, cycle counts, and flag effects.

## Test Suites

### Blargg

| Test name | Status |
|---|---|
| CPU instructions tests (Isolated) | ✅ Passing |
| CPU instructions tests (Integrated) | ✅ Passing |
| Instructions Timing tests | ✅ Passing |

### Mooneye

| Test name | Status |
|---|---|
| Instructions Timing Tests | ✅ Passing |
| Boot registers test | ✅ Passing |
| Boot DIV test | ✅ Passing |
| DAA instruction tests | ✅ Passing |
| POP Timing tests | ✅ Passing |
| PUSH Timing tests | ✅ Passing |
| Interrupt tests | ✅ Passing |
| Timer tests | ✅ Passing |
| MBC1 (No Multicarts) | ✅ Passing |
| MBC2/3 | Not tested |
| OAM DMA tests | ✅ Passing |
| HALT Tests | ✅ Passing |
| PPU Tests | ⏳ In progress... |

---------------------------------------
<div align="center">
  <img align="center" width="352" alt="gbe" src="https://github.com/Axl-91/gbe-rs/blob/main/assets/Blargg.jpg" />
</div>

