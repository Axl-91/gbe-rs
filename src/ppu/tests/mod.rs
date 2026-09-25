use super::*;
const LCDC_ENABLE: u8 = 7;

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

// --- A partir de acá, tests reescritos para stat_line / update_stat_line ---

#[test]
fn stat_line_is_false_when_lcd_disabled_regardless_of_conditions() {
    let mut ppu = Ppu::new();

    ppu.lcdc &= !(1 << LCDC_ENABLE); // LCD apagado
    ppu.mode = PpuMode::HBlank;
    ppu.stat |= 1 << 3; // HBlank-interrupt habilitado
    ppu.ly_eq_lyc = true;
    ppu.stat |= 1 << 6; // LYC-interrupt habilitado

    assert!(!ppu.compute_stat_line());
}

#[test]
fn hblank_stat_interrupt_is_requested_when_enabled() {
    let mut ppu = Ppu::new();

    ppu.mode = PpuMode::HBlank;
    ppu.stat |= 1 << 3;

    assert!(ppu.update_stat_line());
}

#[test]
fn hblank_stat_interrupt_is_not_requested_when_disabled() {
    let mut ppu = Ppu::new();

    ppu.mode = PpuMode::HBlank;

    assert!(!ppu.update_stat_line());
}

#[test]
fn vblank_stat_interrupt_is_requested_when_enabled() {
    let mut ppu = Ppu::new();

    ppu.mode = PpuMode::VBlank;
    ppu.stat |= 1 << 4;

    assert!(ppu.update_stat_line());
}

#[test]
fn vblank_stat_interrupt_is_not_requested_when_disabled() {
    let mut ppu = Ppu::new();

    ppu.mode = PpuMode::VBlank;

    assert!(!ppu.update_stat_line());
}

#[test]
fn oam_search_stat_interrupt_is_requested_when_enabled() {
    let mut ppu = Ppu::new();

    ppu.mode = PpuMode::OamSearch;
    ppu.stat |= 1 << 5;

    assert!(ppu.update_stat_line());
}

#[test]
fn oam_search_stat_interrupt_is_not_requested_when_disabled() {
    let mut ppu = Ppu::new();

    ppu.mode = PpuMode::OamSearch;

    assert!(!ppu.update_stat_line());
}

#[test]
fn drawing_does_not_request_stat_interrupt() {
    let mut ppu = Ppu::new();

    ppu.mode = PpuMode::Drawing;
    ppu.stat |= 1 << 3;
    ppu.stat |= 1 << 4;
    ppu.stat |= 1 << 5;

    assert!(!ppu.update_stat_line());
}

#[test]
fn lyc_stat_interrupt_is_requested_when_enabled() {
    let mut ppu = Ppu::new();

    ppu.stat |= 1 << 6;
    ppu.ly_eq_lyc = true;

    assert!(ppu.update_stat_line());
}

#[test]
fn lyc_stat_interrupt_is_not_requested_when_disabled() {
    let mut ppu = Ppu::new();

    ppu.ly_eq_lyc = true;

    assert!(!ppu.update_stat_line());
}

#[test]
fn lyc_stat_interrupt_is_not_requested_without_a_match() {
    let mut ppu = Ppu::new();

    ppu.stat |= 1 << 6;
    ppu.ly_eq_lyc = false;

    assert!(!ppu.update_stat_line());
}

#[test]
fn stat_line_does_not_rise_again_while_condition_stays_true() {
    // Es el "STAT IRQ Blocking": una vez arriba, no vuelve a subir
    // hasta que la señal combinada caiga a false primero.
    let mut ppu = Ppu::new();

    ppu.stat |= 1 << 6;
    ppu.ly_eq_lyc = true;

    assert!(ppu.update_stat_line()); // primer flanco: sube
    assert!(!ppu.update_stat_line()); // sigue en true: no hay flanco nuevo
    assert!(!ppu.update_stat_line()); // idem, por más veces que se llame
}

#[test]
fn stat_line_rises_again_after_falling() {
    let mut ppu = Ppu::new();

    ppu.stat |= 1 << 6;
    ppu.ly_eq_lyc = true;

    assert!(ppu.update_stat_line()); // sube
    assert!(!ppu.update_stat_line()); // bloqueado

    ppu.ly_eq_lyc = false;
    assert!(!ppu.update_stat_line()); // cae, pero cae no dispara nada

    ppu.ly_eq_lyc = true;
    assert!(ppu.update_stat_line()); // vuelve a subir: nuevo flanco
}

#[test]
fn mode_and_lyc_stat_interrupts_share_the_same_request() {
    // Dos condiciones activas a la vez siguen contando como un solo
    // flanco, no dos interrupts independientes.
    let mut ppu = Ppu::new();

    ppu.mode = PpuMode::HBlank;
    ppu.stat |= 1 << 3;
    ppu.stat |= 1 << 6;
    ppu.ly_eq_lyc = true;

    assert!(ppu.update_stat_line()); // un único flanco
    assert!(!ppu.update_stat_line()); // sigue bloqueado, aunque ambas condiciones sigan activas
}
