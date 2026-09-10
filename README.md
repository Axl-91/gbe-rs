# gbe-rs

A Game Boy DMG emulator written in Rust.

The goal of this project is to build a simple and accurate emulator for the original Game Boy (DMG), starting from the core hardware and gradually adding the components required to run real games.

## Current progress

### CPU
- [x] CPU registers and flags
- [x] Fetch / decode / execute cycle
- [ ] Instruction set
  - [x] NOP
  - [x] INC 8-bit
  - [x] DEC 8-bit
  - [x] LD 8-bit
  - [ ] LD 16-bit
  - [ ] 8-bit arithmetic / logic
  - [ ] 16-bit arithmetic
  - [ ] Rotations / shifts
  - [ ] Bit operations
  - [ ] Control flow
  - [ ] Stack operations
  - [ ] CPU control instructions
- [ ] Instruction timing

### Memory
- [x] Memory bus
- [x] VRAM
- [x] WRAM
- [ ] HRAM
- [ ] I/O registers
- [ ] Interrupt registers

### Cartridge
- [x] Cartridge ROM
- [x] MBC1
- [x] Cartridge RAM

### Hardware
- [ ] Timer
- [ ] Interrupts
- [ ] PPU
- [ ] Input

### Frontend
- [ ] Frontend

## Emulator logic

The emulator works by reproducing the main hardware components of the Game Boy and allowing them to interact with each other.

At the center is the CPU, which fetches instructions from memory, decodes them, and executes them. Memory access is handled through a memory bus, which routes reads and writes to the appropriate hardware component.

<img width="1408" height="768" alt="gbe_logic" src="https://github.com/user-attachments/assets/14a29bc9-92fa-462c-a574-9d1160820e01" />


As development progresses, the bus will also connect the CPU with the remaining hardware components:

```text
Memory Bus
├── Cartridge
├── VRAM
├── WRAM
├── HRAM
├── I/O Registers
├── Timer
├── PPU
├── Input
└── Interrupts
```

The emulator advances the hardware over time by executing CPU instructions and, eventually, consuming the corresponding number of cycles. This allows components such as the PPU and timer to remain synchronized with the CPU.

## Project structure

```text
gbe-rs/
├── src/
│   ├── cpu/
│   │   ├── instruction.rs
│   │   ├── registers.rs
│   │   └── mod.rs
│   │
│   ├── memory/
│   │   ├── bus.rs
│   │   ├── map.rs
│   │   └── mod.rs
│   │
│   ├── cartridge/
│   │   ├── mbc1.rs
│   │   └── mod.rs
│   │
│   └── main.rs
│
├── Cargo.toml
└── README.md
```


------------------


<div align="center">
  <img width="500" height="342" alt="gameboy" src="https://github.com/user-attachments/assets/df020866-0056-465e-9664-b5b50a88f5e4" />
</div>
