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

# Test Suites

## Blargg

| Test name | Status |
|---|---|
| `01-special.gb` | ✅ Passing |
| `02-interrupts.gb` | ✅ Passing |
| `03-op sp,hl.gb` | ✅ Passing |
| `04-op r,imm.gb` | ✅ Passing |
| `05-op rp.gb` | ✅ Passing |
| `06-ld r,r.gb` | ✅ Passing |
| `07-jr,jp,call,ret,rst.gb` | ✅ Passing |
| `08-misc instrs.gb` | ✅ Passing |
| `09-op r,r.gb` | ✅ Passing |
| `10-bit ops.gb` | ✅ Passing |
| `11-op a,(hl).gb` | ✅ Passing |
| `cpu_instrs.gb` | ✅ Passing |
| `halt_bug.gb` | ❌ Failing |
| `instr_timing.gb` | ❌ Failing |

## Mooneye

| Test name | Status |
|---|---|
| `boot/boot_div-dmgABCmgb.gb` | ❌ Failing |
| `boot/boot_hwio-dmgABCmgb.gb` | ❌ Failing |
| `boot/boot_regs-dmgABC.gb` | ✅ Passing |
| `cpu/daa.gb` | ✅ Passing |
| `cpu/halt_ime0_ei.gb` | ⏳ Not tested |
| `cpu/halt_ime0_nointr_timing.gb` | ⏳ Not tested |
| `cpu/halt_ime1_timing.gb` | ✅ Passing |
| `cpu/pop_timing.gb` | ✅ Passing |
| `interrupt/ei_sequence.gb` | ✅ Passing |
| `interrupt/ei_timing.gb` | ✅ Passing |
| `interrupt/ie_push.gb` | ✅ Passing |
| `interrupt/if_ie_registers.gb` | ✅ Passing |
| `interrupt/rapid_di_ei.gb` | ✅ Passing |
| `interrupt/rapid_toggle.gb` | ✅ Passing |
| `interrupt/reti_intr_timing.gb` | ✅ Passing |
| `intr_timing.gb` | ✅ Passing |
| `mbc1/bits_bank1.gb` | ✅ Passing |
| `mbc1/bits_bank2.gb` | ✅ Passing |
| `mbc1/bits_mode.gb` | ⏳ Not tested |
| `mbc1/bits_ramg.gb` | ✅ Passing |
| `timer/div_timing.gb` | ✅ Passing |
| `timer/div_write.gb` | ✅ Passing |
| `timer/tim00_div_trigger.gb` | ✅ Passing |
| `timer/tim00.gb` | ✅ Passing |
| `timer/tim01_div_trigger.gb` | ✅ Passing |
| `timer/tim01.gb` | ✅ Passing |
| `timer/tim10_div_trigger.gb` | ✅ Passing |
| `timer/tim10.gb` | ✅ Passing |
| `timer/tim11_div_trigger.gb` | ✅ Passing |
| `timer/tim11.gb` | ✅ Passing |
| `timer/tima_reload.gb` | ✅ Passing |
| `timer/tima_write_reloading.gb` | ✅ Passing |
| `timer/tma_write_reloading.gb` | ✅ Passing |
