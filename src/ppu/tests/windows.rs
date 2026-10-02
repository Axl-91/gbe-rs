use super::*;

use crate::memory::map::VRAM_START;

const TILE_MAP_START: u16 = VRAM_START + 0x1800;
const TILE_MAP_ALT_START: u16 = VRAM_START + 0x1C00;

const FETCHER_STARTUP_CYCLES: u8 = 6;

fn complete_fetcher_startup(ppu: &mut Ppu) {
    for _ in 0..FETCHER_STARTUP_CYCLES {
        ppu.bg_fetcher.tick();
    }
}

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

#[test]
fn background_context_uses_background_tile_map() {
    let mut ppu = Ppu::new();

    ppu.lcdc &= !(1 << 3);
    ppu.window_active = false;

    ppu.add_fetcher_context();
    complete_fetcher_startup(&mut ppu);

    assert!(ppu.bg_fetcher.tick().is_none());

    match ppu.bg_fetcher.tick() {
        Some(FetcherRequest::ReadVram(address)) => {
            assert_eq!(address, TILE_MAP_START);
        }
        _ => panic!("Expected a background tile map request"),
    }
}

#[test]
fn background_context_uses_alternate_tile_map() {
    let mut ppu = Ppu::new();

    ppu.lcdc |= 1 << 3;
    ppu.window_active = false;

    ppu.add_fetcher_context();
    complete_fetcher_startup(&mut ppu);

    assert!(ppu.bg_fetcher.tick().is_none());

    match ppu.bg_fetcher.tick() {
        Some(FetcherRequest::ReadVram(address)) => {
            assert_eq!(address, TILE_MAP_ALT_START);
        }
        _ => panic!("Expected a background tile map request"),
    }
}

#[test]
fn window_context_uses_window_tile_map() {
    let mut ppu = Ppu::new();

    ppu.window_active = true;
    ppu.lcdc &= !(1 << 6);

    ppu.add_fetcher_context();
    complete_fetcher_startup(&mut ppu);

    assert!(ppu.bg_fetcher.tick().is_none());

    match ppu.bg_fetcher.tick() {
        Some(FetcherRequest::ReadVram(address)) => {
            assert_eq!(address, TILE_MAP_START);
        }
        _ => panic!("Expected a window tile map request"),
    }
}

#[test]
fn window_context_uses_alternate_tile_map() {
    let mut ppu = Ppu::new();

    ppu.window_active = true;
    ppu.lcdc |= 1 << 6;

    ppu.add_fetcher_context();
    complete_fetcher_startup(&mut ppu);

    assert!(ppu.bg_fetcher.tick().is_none());

    match ppu.bg_fetcher.tick() {
        Some(FetcherRequest::ReadVram(address)) => {
            assert_eq!(address, TILE_MAP_ALT_START);
        }
        _ => panic!("Expected a window tile map request"),
    }
}

#[test]
fn background_context_uses_current_scroll_position() {
    let mut ppu = Ppu::new();

    ppu.window_active = false;
    ppu.ly = rand::random();
    ppu.scy = rand::random();
    ppu.scx = rand::random();

    ppu.add_fetcher_context();
    complete_fetcher_startup(&mut ppu);

    assert!(ppu.bg_fetcher.tick().is_none());

    match ppu.bg_fetcher.tick() {
        Some(FetcherRequest::ReadVram(address)) => {
            let y = ppu.ly.wrapping_add(ppu.scy);
            let y_offset = 32 * (y / 8) as u16;

            let x_offset = (ppu.scx / 8) as u16;
            let expected_offset = (y_offset + x_offset) & 0x03FF;

            assert_eq!(address, TILE_MAP_START + expected_offset);
        }
        _ => panic!("Expected a background tile map request"),
    }
}

#[test]
fn window_context_uses_window_line_counter() {
    let mut ppu = Ppu::new();

    ppu.window_active = true;
    ppu.window_line_counter = rand::random();

    ppu.add_fetcher_context();
    complete_fetcher_startup(&mut ppu);

    assert!(ppu.bg_fetcher.tick().is_none());

    match ppu.bg_fetcher.tick() {
        Some(FetcherRequest::ReadVram(address)) => {
            let y_offset = 32 * (ppu.window_line_counter / 8) as u16;

            assert_eq!(address, TILE_MAP_START + y_offset);
        }
        _ => panic!("Expected a window tile map request"),
    }
}

#[test]
fn window_context_ignores_background_scroll() {
    let mut ppu = Ppu::new();

    ppu.window_active = true;
    ppu.window_line_counter = rand::random();

    ppu.scx = rand::random();
    ppu.scy = rand::random();

    ppu.add_fetcher_context();
    complete_fetcher_startup(&mut ppu);

    assert!(ppu.bg_fetcher.tick().is_none());

    match ppu.bg_fetcher.tick() {
        Some(FetcherRequest::ReadVram(address)) => {
            let y_offset = 32 * (ppu.window_line_counter / 8) as u16;

            assert_eq!(address, TILE_MAP_START + y_offset);
        }
        _ => panic!("Expected a window tile map request"),
    }
}

#[test]
fn background_context_uses_current_tile_row() {
    let mut ppu = Ppu::new();

    ppu.ly = rand::random();
    ppu.scy = rand::random();
    ppu.drawing_x = 0;

    ppu.add_fetcher_context();

    complete_fetcher_startup(&mut ppu);

    // Complete TileNumber step.
    ppu.bg_fetcher.tick();

    let tile_number = rand::random();

    match ppu.bg_fetcher.tick() {
        Some(FetcherRequest::ReadVram(_)) => {
            ppu.bg_fetcher.receive(tile_number);
        }
        _ => panic!("Expected a tile number request"),
    }

    // Complete TileDataLow step.
    ppu.bg_fetcher.tick();

    let row = ppu.ly.wrapping_add(ppu.scy) % 8;
    let expected_address = VRAM_START + tile_number as u16 * 16 + row as u16 * 2;

    match ppu.bg_fetcher.tick() {
        Some(FetcherRequest::ReadVram(address)) => {
            assert_eq!(address, expected_address);
        }
        _ => panic!("Expected a tile data low request"),
    }
}

#[test]
fn window_context_uses_current_tile_row() {
    let mut ppu = Ppu::new();

    ppu.lcdc |= 1 << 5;
    ppu.window_active = true;
    ppu.window_line_counter = rand::random();
    ppu.drawing_x = 0;

    ppu.add_fetcher_context();

    complete_fetcher_startup(&mut ppu);

    // Complete TileNumber step.
    ppu.bg_fetcher.tick();

    let tile_number = rand::random();

    match ppu.bg_fetcher.tick() {
        Some(FetcherRequest::ReadVram(_)) => {
            ppu.bg_fetcher.receive(tile_number);
        }
        _ => panic!("Expected a tile number request"),
    }

    // Complete TileDataLow step.
    ppu.bg_fetcher.tick();

    let row = ppu.window_line_counter % 8;
    let expected_address = VRAM_START + tile_number as u16 * 16 + row as u16 * 2;

    match ppu.bg_fetcher.tick() {
        Some(FetcherRequest::ReadVram(address)) => {
            assert_eq!(address, expected_address);
        }
        _ => panic!("Expected a tile data low request"),
    }
}

#[test]
fn starting_window_clears_fifo() {
    let mut ppu = Ppu::new();

    ppu.lcdc |= 1 << 5;
    ppu.ly = ppu.wy;
    ppu.wx = 7;
    ppu.drawing_x = 0;

    let low = rand::random();
    let high = rand::random();

    ppu.push_into_fifo(low, high);

    assert!(!ppu.bg_fifo.is_empty());

    ppu.tick_fetcher();

    assert!(ppu.bg_fifo.is_empty());
}

#[test]
fn starting_window_resets_fetcher() {
    let mut ppu = Ppu::new();

    ppu.lcdc |= 1 << 5;
    ppu.ly = ppu.wy;
    ppu.wx = 7;
    ppu.drawing_x = 0;

    // Advance the fetcher until it is about to request the tile number.
    complete_fetcher_startup(&mut ppu);
    ppu.bg_fetcher.tick();

    // Starting the window must reset the fetcher before it continues.
    ppu.tick_fetcher();

    // After the reset, the fetcher is back in its startup phase.
    // The first tick of that phase does not generate a VRAM request.
    assert!(ppu.bg_fetcher.tick().is_none());
}

#[test]
fn window_activation_switches_fetcher_to_window_context() {
    let mut ppu = Ppu::new();

    ppu.lcdc |= 1 << 5;
    ppu.ly = ppu.wy;
    ppu.wx = 7;
    ppu.drawing_x = 0;

    ppu.tick_fetcher();

    assert!(ppu.window_active);

    for _ in 0..(FETCHER_STARTUP_CYCLES - 1) {
        ppu.bg_fetcher.tick();
    }

    ppu.bg_fetcher.tick();

    match ppu.bg_fetcher.tick() {
        Some(FetcherRequest::ReadVram(address)) => {
            assert_eq!(address, TILE_MAP_START);
        }
        _ => panic!("Expected a window tile map request"),
    }
}
