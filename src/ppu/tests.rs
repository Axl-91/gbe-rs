use super::*;

#[test]
fn ppu_starts_in_oam_search() {
    let ppu = Ppu::new();

    assert!(matches!(ppu.mode, PpuMode::OamSearch));
    assert_eq!(ppu.mode_cycles, 0);
    assert_eq!(ppu.ly, 0);
}

#[test]
fn oam_search_transitions_to_drawing() {
    let mut ppu = Ppu::new();

    for _ in 0..80 {
        ppu.tick();
    }

    assert!(matches!(ppu.mode, PpuMode::Drawing));
    assert_eq!(ppu.mode_cycles, 0);
}

#[test]
fn drawing_transitions_to_hblank() {
    let mut ppu = Ppu::new();

    for _ in 0..80 {
        ppu.tick();
    }

    for _ in 0..172 {
        ppu.tick();
    }

    assert!(matches!(ppu.mode, PpuMode::HBlank));
    assert_eq!(ppu.mode_cycles, 0);
}

#[test]
fn hblank_increments_ly() {
    let mut ppu = Ppu::new();

    for _ in 0..80 {
        ppu.tick();
    }

    for _ in 0..172 {
        ppu.tick();
    }

    for _ in 0..204 {
        ppu.tick();
    }

    assert_eq!(ppu.ly, 1);
    assert!(matches!(ppu.mode, PpuMode::OamSearch));
}

#[test]
fn last_visible_line_enters_vblank() {
    let mut ppu = Ppu::new();

    ppu.ly = 143;
    ppu.mode = PpuMode::HBlank;
    ppu.mode_cycles = 0;

    for _ in 0..204 {
        ppu.tick();
    }

    assert_eq!(ppu.ly, 144);
    assert!(matches!(ppu.mode, PpuMode::VBlank));
}

#[test]
fn vblank_advances_to_next_line() {
    let mut ppu = Ppu::new();

    ppu.ly = 144;
    ppu.mode = PpuMode::VBlank;
    ppu.mode_cycles = 0;

    for _ in 0..456 {
        ppu.tick();
    }

    assert_eq!(ppu.ly, 145);
    assert!(matches!(ppu.mode, PpuMode::VBlank));
}

#[test]
fn last_vblank_line_returns_to_first_line() {
    let mut ppu = Ppu::new();

    ppu.ly = 153;
    ppu.mode = PpuMode::VBlank;
    ppu.mode_cycles = 0;

    for _ in 0..456 {
        ppu.tick();
    }

    assert_eq!(ppu.ly, 0);
    assert!(matches!(ppu.mode, PpuMode::OamSearch));
}
