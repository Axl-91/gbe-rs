use super::*;

use rand::RngExt;

const PIXELS_PER_TILE: usize = 8;
const MAX_TILES: usize = 8;

const STARTUP_DELAY: u8 = 6;

fn complete_start_up_phase(fetcher: &mut BgFetcher) {
    for _ in 0..STARTUP_DELAY {
        fetcher.tick();
    }
}

mod bg_fifo;
mod fetcher;
mod memory;
mod registers;
mod sprite_fetcher;
mod sprites;
mod windows;

#[test]
fn ly_resets_during_line_153_before_vblank_ends() {
    let mut ppu = Ppu::new();

    let max_cycles = SCANLINE_CYCLES as usize * TOTAL_LINES as usize * 2;
    let mut cycles = 0;

    while (ppu.ly != TOTAL_LINES - 1 || !matches!(ppu.mode, PpuMode::VBlank)) && cycles < max_cycles
    {
        ppu.tick();
        cycles += 1;
    }

    assert_eq!(
        ppu.ly,
        TOTAL_LINES - 1,
        "PPU did not reach line 153 within the cycle limit"
    );
    assert!(
        matches!(ppu.mode, PpuMode::VBlank),
        "PPU did not enter VBlank on line 153"
    );

    for _ in 0..LY_RESET_CYCLE {
        ppu.tick();
    }

    assert_eq!(ppu.ly, 0);
    assert!(matches!(ppu.mode, PpuMode::VBlank));

    while matches!(ppu.mode, PpuMode::VBlank) {
        ppu.tick();
    }

    assert_eq!(ppu.ly, 0);
    assert!(matches!(ppu.mode, PpuMode::OamSearch));
}
