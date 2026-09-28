use super::*;

use rand::RngExt;

const PIXELS_PER_TILE: usize = 8;
const MAX_TILES: usize = 8;

mod fetcher;
mod fifo;
mod memory;
mod registers;

#[test]
fn window_starts_when_wy_is_reached() {
    let mut ppu = Ppu::new();

    ppu.lcdc |= 1 << 5;
    ppu.wy = 10;
    ppu.ly = 9;
    ppu.wx = 7;
    ppu.drawing_x = 0;

    assert!(!ppu.can_start_window());

    ppu.ly = 10;

    assert!(ppu.can_start_window());
}

#[test]
fn window_starts_at_wx_minus_seven() {
    let mut ppu = Ppu::new();

    ppu.lcdc |= 1 << 5;
    ppu.ly = ppu.wy;
    ppu.wx = 15;

    ppu.drawing_x = 7;
    assert!(!ppu.can_start_window());

    ppu.drawing_x = 8;
    assert!(ppu.can_start_window());
}

#[test]
fn window_starts_at_screen_edge_when_wx_is_less_than_seven() {
    let mut ppu = Ppu::new();

    ppu.lcdc |= 1 << 5;
    ppu.ly = ppu.wy;
    ppu.wx = 3;
    ppu.drawing_x = 0;

    assert!(ppu.can_start_window());
}

#[test]
fn window_does_not_start_before_wy() {
    let mut ppu = Ppu::new();

    ppu.lcdc |= 1 << 5;
    ppu.wy = 20;
    ppu.ly = 19;
    ppu.wx = 7;
    ppu.drawing_x = 0;

    assert!(!ppu.can_start_window());
}

#[test]
fn background_is_used_before_window_starts() {
    let mut ppu = Ppu::new();

    ppu.lcdc |= 1 << 5;
    ppu.wy = 20;
    ppu.ly = 19;
    ppu.wx = 7;
    ppu.drawing_x = 0;

    assert!(!ppu.can_start_window());
}
