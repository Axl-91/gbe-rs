use super::*;
use crate::memory::map::VRAM_START;
use crate::ppu::sprite_fetcher::SpriteFetcherRequest;
use rand::RngExt;

const SPRITE_Y_OFFSET: u8 = 16;
const SPRITE_HEIGHT_8X8: u8 = 8;
const SPRITE_HEIGHT_8X16: u8 = 16;
const TILE_SIZE_BYTES: u16 = 16;
const ATTRIBUTE_Y_FLIP: u8 = 6;

fn tick_until_request(fetcher: &mut SpriteFetcher) -> SpriteFetcherRequest {
    loop {
        if let Some(request) = fetcher.tick() {
            return request;
        }
    }
}

#[test]
fn tile_number_consumes_two_cycles_without_request() {
    let mut fetcher = SpriteFetcher::new();

    let tick = fetcher.tick();
    assert!(tick.is_none());

    let tick = fetcher.tick();
    assert!(tick.is_none());
}

#[test]
fn tile_data_low_requests_correct_address() {
    let mut rng = rand::rng();
    let mut fetcher = SpriteFetcher::new();

    let ly = rng.random_range(0..VISIBLE_LINES);
    let row = rng.random_range(0..SPRITE_HEIGHT_8X8);
    let tile = rng.random();

    let sprite = Sprite {
        x: rng.random(),
        y: ly + SPRITE_Y_OFFSET - row,
        tile,
        attributes: 0,
    };

    fetcher.add_context(sprite, ly, SPRITE_HEIGHT_8X8);

    let request = tick_until_request(&mut fetcher);

    let expected_address = VRAM_START + tile as u16 * TILE_SIZE_BYTES + row as u16 * 2;

    assert!(matches!(
        request,
        SpriteFetcherRequest::ReadVram(address)
            if address == expected_address
    ));
}

#[test]
fn tile_data_high_requests_next_byte() {
    let mut rng = rand::rng();
    let mut fetcher = SpriteFetcher::new();

    let ly = rng.random_range(0..VISIBLE_LINES);
    let row = rng.random_range(0..SPRITE_HEIGHT_8X8);
    let tile = rng.random();

    let sprite = Sprite {
        x: rng.random(),
        y: ly + SPRITE_Y_OFFSET - row,
        tile,
        attributes: 0,
    };

    fetcher.add_context(sprite, ly, SPRITE_HEIGHT_8X8);

    let _ = tick_until_request(&mut fetcher);
    fetcher.receive(rng.random());

    let request = tick_until_request(&mut fetcher);

    let expected_address = VRAM_START + tile as u16 * TILE_SIZE_BYTES + row as u16 * 2 + 1;

    assert!(matches!(
        request,
        SpriteFetcherRequest::ReadVram(address)
            if address == expected_address
    ));
}

#[test]
fn receive_stores_low_and_high_bytes() {
    let mut rng = rand::rng();
    let mut fetcher = SpriteFetcher::new();

    let ly = rng.random_range(0..VISIBLE_LINES);
    let row = rng.random_range(0..SPRITE_HEIGHT_8X8);

    let sprite = Sprite {
        x: rng.random(),
        y: ly + SPRITE_Y_OFFSET - row,
        tile: rng.random(),
        attributes: 0,
    };

    let low = rng.random();
    let high = rng.random();

    fetcher.add_context(sprite, ly, SPRITE_HEIGHT_8X8);

    let _ = tick_until_request(&mut fetcher);
    fetcher.receive(low);

    let _ = tick_until_request(&mut fetcher);
    fetcher.receive(high);

    let request = tick_until_request(&mut fetcher);

    assert!(matches!(
        request,
        SpriteFetcherRequest::Push {
            low: actual_low,
            high: actual_high,
        } if actual_low == low && actual_high == high
    ));
}

#[test]
fn push_happens_immediately() {
    let mut rng = rand::rng();
    let mut fetcher = SpriteFetcher::new();

    let ly = rng.random_range(0..VISIBLE_LINES);
    let row = rng.random_range(0..SPRITE_HEIGHT_8X8);

    let sprite = Sprite {
        x: rng.random(),
        y: ly + SPRITE_Y_OFFSET - row,
        tile: rng.random(),
        attributes: 0,
    };

    let low = rng.random();
    let high = rng.random();

    fetcher.add_context(sprite, ly, SPRITE_HEIGHT_8X8);

    let _ = tick_until_request(&mut fetcher);
    fetcher.receive(low);

    let _ = tick_until_request(&mut fetcher);
    fetcher.receive(high);

    let request = fetcher.tick();

    assert!(matches!(
        request,
        Some(SpriteFetcherRequest::Push {
            low: actual_low,
            high: actual_high,
        }) if actual_low == low && actual_high == high
    ));
}

#[test]
fn complete_push_starts_next_tile_number_step() {
    let mut fetcher = SpriteFetcher::new();

    fetcher.complete_push();

    assert!(fetcher.tick().is_none());
    assert!(fetcher.tick().is_none());
}

#[test]
fn eight_by_eight_sprite_requests_correct_row() {
    let mut rng = rand::rng();
    let mut fetcher = SpriteFetcher::new();

    let ly = rng.random_range(0..VISIBLE_LINES);
    let row = rng.random_range(0..SPRITE_HEIGHT_8X8);
    let tile = rng.random();

    let sprite = Sprite {
        x: rng.random(),
        y: ly + SPRITE_Y_OFFSET - row,
        tile,
        attributes: 0,
    };

    fetcher.add_context(sprite, ly, SPRITE_HEIGHT_8X8);

    let request = tick_until_request(&mut fetcher);

    let expected_address = VRAM_START + tile as u16 * TILE_SIZE_BYTES + row as u16 * 2;

    assert!(matches!(
        request,
        SpriteFetcherRequest::ReadVram(address)
            if address == expected_address
    ));
}

#[test]
fn eight_by_eight_y_flip_requests_flipped_row() {
    let mut rng = rand::rng();
    let mut fetcher = SpriteFetcher::new();

    let ly = rng.random_range(0..VISIBLE_LINES);
    let row = rng.random_range(0..SPRITE_HEIGHT_8X8);
    let tile = rng.random();

    let sprite = Sprite {
        x: rng.random(),
        y: ly + SPRITE_Y_OFFSET - row,
        tile,
        attributes: 1 << ATTRIBUTE_Y_FLIP,
    };

    fetcher.add_context(sprite, ly, SPRITE_HEIGHT_8X8);

    let request = tick_until_request(&mut fetcher);

    let flipped_row = SPRITE_HEIGHT_8X8 - 1 - row;
    let expected_address = VRAM_START + tile as u16 * TILE_SIZE_BYTES + flipped_row as u16 * 2;

    assert!(matches!(
        request,
        SpriteFetcherRequest::ReadVram(address)
            if address == expected_address
    ));
}

#[test]
fn eight_by_sixteen_sprite_requests_first_tile() {
    let mut rng = rand::rng();
    let mut fetcher = SpriteFetcher::new();

    let ly = rng.random_range(0..VISIBLE_LINES);
    let row = rng.random_range(0..SPRITE_HEIGHT_8X8);
    let tile = rng.random::<u8>() & !1;

    let sprite = Sprite {
        x: rng.random(),
        y: ly + SPRITE_Y_OFFSET - row,
        tile,
        attributes: 0,
    };

    fetcher.add_context(sprite, ly, SPRITE_HEIGHT_8X16);

    let request = tick_until_request(&mut fetcher);

    let expected_address = VRAM_START + tile as u16 * TILE_SIZE_BYTES + row as u16 * 2;

    assert!(matches!(
        request,
        SpriteFetcherRequest::ReadVram(address)
            if address == expected_address
    ));
}

#[test]
fn eight_by_sixteen_sprite_requests_second_tile() {
    let mut rng = rand::rng();
    let mut fetcher = SpriteFetcher::new();

    let ly = rng.random_range(0..VISIBLE_LINES);
    let row = rng.random_range(SPRITE_HEIGHT_8X8..SPRITE_HEIGHT_8X16);
    let tile = rng.random::<u8>() & !1;

    let sprite = Sprite {
        x: rng.random(),
        y: ly + SPRITE_Y_OFFSET - row,
        tile,
        attributes: 0,
    };

    fetcher.add_context(sprite, ly, SPRITE_HEIGHT_8X16);

    let request = tick_until_request(&mut fetcher);

    let tile_row = row - SPRITE_HEIGHT_8X8;
    let expected_address = VRAM_START + (tile as u16 + 1) * TILE_SIZE_BYTES + tile_row as u16 * 2;

    assert!(matches!(
        request,
        SpriteFetcherRequest::ReadVram(address)
            if address == expected_address
    ));
}

#[test]
fn eight_by_sixteen_y_flip_requests_correct_tile_and_row() {
    let mut rng = rand::rng();
    let mut fetcher = SpriteFetcher::new();

    let ly = rng.random_range(0..VISIBLE_LINES);
    let row = rng.random_range(0..SPRITE_HEIGHT_8X16);
    let tile = rng.random::<u8>() & !1;

    let sprite = Sprite {
        x: rng.random(),
        y: ly + SPRITE_Y_OFFSET - row,
        tile,
        attributes: 1 << ATTRIBUTE_Y_FLIP,
    };

    fetcher.add_context(sprite, ly, SPRITE_HEIGHT_8X16);

    let request = tick_until_request(&mut fetcher);

    let flipped_row = SPRITE_HEIGHT_8X16 - 1 - row;
    let tile_offset = if flipped_row < SPRITE_HEIGHT_8X8 {
        0
    } else {
        1
    };
    let tile_row = flipped_row % SPRITE_HEIGHT_8X8;

    let expected_address =
        VRAM_START + (tile as u16 + tile_offset) * TILE_SIZE_BYTES + tile_row as u16 * 2;

    assert!(matches!(
        request,
        SpriteFetcherRequest::ReadVram(address)
            if address == expected_address
    ));
}
