use crate::ppu::fetcher::{Fetcher, FetcherRequest};

#[test]
fn fetcher_requests_tile_number_first() {
    let mut fetcher = Fetcher::new();

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
