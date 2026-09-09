# gbe-rs

A Game Boy DMG emulator written in Rust.

The goal of this project is to build a simple and accurate emulator for the original Game Boy (DMG), starting from the core hardware and gradually adding the components required to run real games.

## Current progress

### CPU
- [x] CPU registers and flags
- [ ] Fetch / decode / execute cycle
- [ ] Instruction set

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

--------

<div align="center">
  <img width="500" height="342" alt="gameboy" src="https://github.com/user-attachments/assets/df020866-0056-465e-9664-b5b50a88f5e4" />
</div>
