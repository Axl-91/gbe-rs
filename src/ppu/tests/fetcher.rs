use crate::ppu::fetcher::{Fetcher, FetcherRequest};

const STARTUP_DELAY: u8 = 6;
const TILE_DATA_SIGNED_START: u16 = 0x9000;

fn complete_start_up_phase(fetcher: &mut Fetcher) {
    for _ in 0..STARTUP_DELAY {
        fetcher.tick();
    }
}

#[test]
fn fetcher_requests_tile_number_first() {
    let mut fetcher = Fetcher::new();
    complete_start_up_phase(&mut fetcher);

    let address = crate::memory::map::VRAM_START;
    let row = 0;

    fetcher.add_context(address, row, true);

    assert!(fetcher.tick().is_none());

    match fetcher.tick() {
        Some(FetcherRequest::ReadVram(request_address)) => {
            assert_eq!(request_address, address);
        }
        _ => panic!("Expected a tile number request"),
    }
}

#[test]
fn fetcher_requests_tile_data_low_after_tile_number() {
    let mut fetcher = Fetcher::new();
    complete_start_up_phase(&mut fetcher);

    let tile_map_address = crate::memory::map::VRAM_START;
    let tile_number = 0;
    let row = 0;

    fetcher.add_context(tile_map_address, row, true);

    fetcher.tick();
    fetcher.tick();

    fetcher.receive(tile_number);

    fetcher.tick();

    match fetcher.tick() {
        Some(FetcherRequest::ReadVram(_)) => {}
        _ => panic!("Expected a tile data low request"),
    }
}

#[test]
fn fetcher_requests_tile_data_high_after_low() {
    let mut fetcher = Fetcher::new();
    complete_start_up_phase(&mut fetcher);

    let tile_map_address = crate::memory::map::VRAM_START;
    let tile_number = 0;
    let row = 0;

    fetcher.add_context(tile_map_address, row, true);

    fetcher.tick();
    fetcher.tick();
    fetcher.receive(tile_number);

    fetcher.tick();
    fetcher.tick();
    fetcher.receive(0);

    fetcher.tick();

    match fetcher.tick() {
        Some(FetcherRequest::ReadVram(_)) => {}
        _ => panic!("Expected a tile data high request"),
    }
}

#[test]
fn fetcher_pushes_received_tile_data() {
    let mut fetcher = Fetcher::new();
    complete_start_up_phase(&mut fetcher);

    let tile_map_address = crate::memory::map::VRAM_START;
    let tile_number = 0;
    let row = 0;

    fetcher.add_context(tile_map_address, row, true);

    fetcher.tick();
    fetcher.tick();
    fetcher.receive(tile_number);

    fetcher.tick();
    fetcher.tick();
    fetcher.receive(0x12);

    fetcher.tick();
    fetcher.tick();
    fetcher.receive(0x34);

    fetcher.tick();

    match fetcher.tick() {
        Some(FetcherRequest::Push { low, high }) => {
            assert_eq!(low, 0x12);
            assert_eq!(high, 0x34);
        }
        _ => panic!("Expected a push request"),
    }
}

#[test]
fn fetcher_uses_unsigned_tile_data_addressing() {
    let mut fetcher = Fetcher::new();
    complete_start_up_phase(&mut fetcher);

    let tile_map_address = crate::memory::map::VRAM_START;
    let tile_number = rand::random::<u8>();
    let row = rand::random::<u8>() % 8;

    fetcher.add_context(tile_map_address, row, true);

    fetcher.tick();
    fetcher.tick();
    fetcher.receive(tile_number);

    fetcher.tick();

    match fetcher.tick() {
        Some(FetcherRequest::ReadVram(address)) => {
            let expected =
                crate::memory::map::VRAM_START + tile_number as u16 * 16 + row as u16 * 2;

            assert_eq!(address, expected);
        }
        _ => panic!("Expected a tile data low request"),
    }
}

#[test]
fn fetcher_uses_signed_tile_data_addressing() {
    let mut fetcher = Fetcher::new();
    complete_start_up_phase(&mut fetcher);

    let tile_map_address = crate::memory::map::VRAM_START;
    let tile_number = rand::random::<u8>();
    let row = rand::random::<u8>() % 8;

    fetcher.add_context(tile_map_address, row, false);

    fetcher.tick();
    fetcher.tick();
    fetcher.receive(tile_number);

    fetcher.tick();

    match fetcher.tick() {
        Some(FetcherRequest::ReadVram(address)) => {
            let signed_tile = tile_number as i8 as i16;
            let expected =
                (TILE_DATA_SIGNED_START as i16 + signed_tile * 16 + row as i16 * 2) as u16;

            assert_eq!(address, expected);
        }
        _ => panic!("Expected a tile data low request"),
    }
}

#[test]
fn fetcher_uses_tile_row_when_fetching_tile_data() {
    let mut fetcher = Fetcher::new();
    complete_start_up_phase(&mut fetcher);

    let tile_map_address = crate::memory::map::VRAM_START;
    let tile_number = rand::random::<u8>();
    let row = rand::random::<u8>() % 8;

    fetcher.add_context(tile_map_address, row, true);

    fetcher.tick();
    fetcher.tick();
    fetcher.receive(tile_number);

    fetcher.tick();

    match fetcher.tick() {
        Some(FetcherRequest::ReadVram(address)) => {
            let expected =
                crate::memory::map::VRAM_START + tile_number as u16 * 16 + row as u16 * 2;

            assert_eq!(address, expected);
        }
        _ => panic!("Expected a tile data low request"),
    }
}

#[test]
fn fetcher_uses_background_tile_map_address() {
    let mut fetcher = Fetcher::new();
    complete_start_up_phase(&mut fetcher);

    let tile_map_address = crate::memory::map::VRAM_START;
    let row = rand::random::<u8>() % 8;

    fetcher.add_context(tile_map_address, row, true);

    fetcher.tick();

    match fetcher.tick() {
        Some(FetcherRequest::ReadVram(address)) => {
            assert_eq!(address, tile_map_address);
        }
        _ => panic!("Expected a tile map request"),
    }
}

#[test]
fn fetcher_uses_window_tile_map_address() {
    let mut fetcher = Fetcher::new();
    complete_start_up_phase(&mut fetcher);

    let tile_map_address = crate::memory::map::VRAM_START + 0x400;
    let row = rand::random::<u8>() % 8;

    fetcher.add_context(tile_map_address, row, true);

    fetcher.tick();

    match fetcher.tick() {
        Some(FetcherRequest::ReadVram(address)) => {
            assert_eq!(address, tile_map_address);
        }
        _ => panic!("Expected a tile map request"),
    }
}
