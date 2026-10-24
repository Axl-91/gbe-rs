<div align="center">
  <img align="center" width="704" alt="kaniboy" src="https://github.com/Axl-91/kaniboy/blob/main/assets/kaniboy.png" />
</div>

#

A Game Boy DMG emulator written in Rust.

The goal of this project is to build a simple and accurate emulator for the original Game Boy (DMG), starting from the core hardware and gradually adding the components required to run real games.

## Status

KaniBoy is an active Game Boy emulator project focused on accurate DMG hardware emulation.

The core DMG hardware is implemented, including:

- CPU and instruction timing
- Memory and interrupts
- Cartridge ROM, MBCs, RAM, and battery-backed saves
- PPU and rendering
- APU and audio output
- Timer
- Joypad input
- Frontend rendering and input

The project is still under active development, with ongoing work focused on improving hardware accuracy, game compatibility, and the overall emulator experience.

## Emulator architecture

The emulator reproduces the main hardware components of the Game Boy and allows them to interact through a shared memory bus and a cycle-based execution model.

At the center is the CPU, which fetches instructions from memory, decodes them, and executes them. Memory access is handled through the memory bus, which routes reads and writes to the appropriate hardware component, such as the cartridge, RAM, PPU, timer, serial interface, or joypad.

<img width="1408" alt="gbe_logic" src="https://github.com/Axl-91/kaniboy/blob/main/assets/diagram.jpg" />

The emulator advances the hardware over time using the CPU's clock cycles. Each executed instruction consumes a specific number of cycles, and the corresponding hardware components are advanced accordingly. This keeps components such as the PPU, timer, DMA controller, and serial interface synchronized with the CPU.

Hardware components are responsible for their own internal state and timing, while the emulator coordinates their execution and the communication between them. This approach keeps the different parts of the emulator modular while allowing them to behave as a single system.

## Requirements (Linux)

- glibc 2.35 or newer (depends on the build machine)
- ALSA (`libasound2`) for audio
- A Wayland or X11 session with Vulkan or OpenGL drivers

## Controls

| Key | Game Boy button |
|---|---|
| `W` `A` `S` `D` | D-pad (up, left, down, right) |
| `J` | A |
| `K` | B |
| `Enter` | Start |
| `Left Shift` | Select |
| `F11` | Fullscreen Mode |
| `Esc` | Quit |

## Running

### Run the emulator

To run the emulator with a Game Boy ROM, without a boot ROM:

```bash
just run-release <PATH_TO_ROM>
```

For example:

```bash
just run-release ./roms/tetris.gb
```

To run the emulator with a boot ROM, use the `run-boot` recipe, providing the boot ROM path first and the game ROM path second:

```bash
just run-boot <PATH_TO_BOOT_ROM> <PATH_TO_ROM>
```

For example, using the Bootix DMG boot ROM included in the repository:

```bash
just run-boot ./bootrom/bootix_dmg.bin ./roms/tetris.gb
```

The boot ROM is optional. When running without one, the emulator starts directly from the post-boot state.

### Run the tests

Run the project's own tests with:

```bash
just test
```

### Run test suites

The repository includes several Game Boy test suites that can be run using:

```bash
./test_suites/run.sh
```

To run a specific test suite:

```bash
./test_suites/run.sh --suite <SUITE>
```

For example:

```bash
./test_suites/run.sh --suite blargg
```

## References

- [Pan Docs](https://gbdev.io/pandocs/) — The single most comprehensive technical reference for Game Boy hardware behavior, covering the CPU, memory map, PPU, timers, interrupts, and more.
- [Game Boy: Complete Technical Reference](https://gekkio.fi/files/gb-docs/gbctr.pdf) — Gekkio's in-depth reference (PDF), the primary source for hardware quirks and cycle-accurate behavior.
- [Game Boy Opcode Table](https://gbdev.io/gb-opcodes/optables/) — Instruction reference with opcodes, cycle counts, and flag effects.

### Acknowledgements

- [Bootix](https://github.com/Ashiepaws/Bootix) by Ashiepaws — Custom DMG/MGB boot ROM used for testing. Licensed under CC0-1.0.
- [Is That a Demo in Your Pocket?](https://demozoo.org/productions/142074/) — Game Boy demo by Snorpung, featuring music by Nordloef.

## Test Suites

I use several test suites to verify the emulator’s functionality. The passing tests from each suite are included in the repository and run in CI:

- [Blargg’s tests](https://github.com/Axl-91/kaniboy/tree/main/test_suites/blargg) (15 tests)
- [Mooneye tests](https://github.com/Axl-91/kaniboy/tree/main/test_suites/mooneye) (~77 tests)
- [GBMicrotests](https://github.com/Axl-91/kaniboy/tree/main/test_suites/microtests) (~90 tests)

---------------------------------------
<div align="center">
  <img align="center" width="352" alt="gbe" src="https://github.com/Axl-91/kaniboy/blob/main/assets/Blargg.jpg" />
</div>

