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

#[test]
fn lyc_match_is_detected_only_when_entering_match() {
    let mut ppu = Ppu::new();

    ppu.lyc = ppu.ly;

    assert!(ppu.update_lyc_match());
    assert!(!ppu.update_lyc_match());

    ppu.lyc = ppu.ly.wrapping_add(1);

    assert!(!ppu.update_lyc_match());

    ppu.lyc = ppu.ly;

    assert!(ppu.update_lyc_match());
}

#[test]
fn hblank_stat_interrupt_is_requested_when_enabled() {
    let mut ppu = Ppu::new();

    ppu.mode = PpuMode::HBlank;
    ppu.stat |= 1 << 3;

    assert!(ppu.stat_interrupt_requested(false));
}

#[test]
fn hblank_stat_interrupt_is_not_requested_when_disabled() {
    let mut ppu = Ppu::new();

    ppu.mode = PpuMode::HBlank;

    assert!(!ppu.stat_interrupt_requested(false));
}

#[test]
fn vblank_stat_interrupt_is_requested_when_enabled() {
    let mut ppu = Ppu::new();

    ppu.mode = PpuMode::VBlank;
    ppu.stat |= 1 << 4;

    assert!(ppu.stat_interrupt_requested(false));
}

#[test]
fn vblank_stat_interrupt_is_not_requested_when_disabled() {
    let mut ppu = Ppu::new();

    ppu.mode = PpuMode::VBlank;

    assert!(!ppu.stat_interrupt_requested(false));
}

#[test]
fn oam_search_stat_interrupt_is_requested_when_enabled() {
    let mut ppu = Ppu::new();

    ppu.mode = PpuMode::OamSearch;
    ppu.stat |= 1 << 5;

    assert!(ppu.stat_interrupt_requested(false));
}

#[test]
fn oam_search_stat_interrupt_is_not_requested_when_disabled() {
    let mut ppu = Ppu::new();

    ppu.mode = PpuMode::OamSearch;

    assert!(!ppu.stat_interrupt_requested(false));
}

#[test]
fn drawing_does_not_request_stat_interrupt() {
    let mut ppu = Ppu::new();

    ppu.mode = PpuMode::Drawing;
    ppu.stat |= 1 << 3;
    ppu.stat |= 1 << 4;
    ppu.stat |= 1 << 5;

    assert!(!ppu.stat_interrupt_requested(false));
}

#[test]
fn lyc_stat_interrupt_is_requested_when_enabled() {
    let mut ppu = Ppu::new();

    ppu.stat |= 1 << 6;

    assert!(ppu.stat_interrupt_requested(true));
}

#[test]
fn lyc_stat_interrupt_is_not_requested_when_disabled() {
    let ppu = Ppu::new();

    assert!(!ppu.stat_interrupt_requested(true));
}

#[test]
fn lyc_stat_interrupt_is_not_requested_without_new_match() {
    let mut ppu = Ppu::new();

    ppu.stat |= 1 << 6;

    assert!(!ppu.stat_interrupt_requested(false));
}

#[test]
fn mode_and_lyc_stat_interrupts_share_the_same_request() {
    let mut ppu = Ppu::new();

    ppu.mode = PpuMode::HBlank;
    ppu.stat |= 1 << 3;
    ppu.stat |= 1 << 6;

    assert!(ppu.stat_interrupt_requested(true));
}
