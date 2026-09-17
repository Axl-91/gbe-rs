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

## Instruction reference

For a complete reference of the Game Boy CPU instruction set, see the [Game Boy CPU Instruction Set](https://gbdev.io/gb-opcodes/optables/).

## Project structure

```text
gbe-rs/
├── src/
│   ├── cartridge/
│   │   ├── mbc1.rs
│   │   └── mod.rs
│   ├── cpu/
│   │   ├── tests/
│   │   ├── arithmetic.rs
│   │   ├── cb.rs
│   │   ├── control.rs
│   │   ├── instruction.rs
│   │   ├── interrupt.rs
│   │   ├── load.rs
│   │   ├── mod.rs
│   │   ├── registers.rs
│   │   ├── rotation.rs
│   │   ├── stack.rs
│   │   └── t_cycles.rs
│   ├── memory/
│   │   ├── bus.rs
│   │   ├── map.rs
│   │   └── mod.rs
│   ├── timer/
│   │   ├── mod.rs
│   │   └── tests.rs
│   ├── lib.rs
│   └── main.rs
├── Cargo.lock
├── Cargo.toml
├── LICENSE
└── README.md
```


------------------


<div align="center">
  <img width="500" height="342" alt="gameboy" src="https://github.com/user-attachments/assets/df020866-0056-465e-9664-b5b50a88f5e4" />
</div>
