use super::*;

#[test]
fn new_fifo_is_empty() {
    let fifo = BgFifo::new();

    assert!(fifo.is_empty());
}

#[test]
fn push_and_pop_preserve_order() {
    let mut fifo = BgFifo::new();

    let mut rng = rand::rng();
    let pixels: Vec<u8> = (0..rng.random_range(1..=MAX_TILES * PIXELS_PER_TILE))
        .map(|_| rng.random_range(0..=3))
        .collect();

    for pixel in &pixels {
        fifo.push(*pixel);
    }

    for pixel in pixels {
        assert_eq!(fifo.pop(), Some(pixel));
    }

    assert!(fifo.is_empty());
}

#[test]
fn pop_empty_fifo_returns_none() {
    let mut fifo = BgFifo::new();

    assert_eq!(fifo.pop(), None);
}

#[test]
fn push_tile_adds_eight_pixels() {
    let mut fifo = BgFifo::new();

    let mut rng = rand::rng();
    let low: u8 = rng.random();
    let high: u8 = rng.random();

    fifo.push_tile(low, high);
}

#[test]
fn push_tile_decodes_pixels_in_order() {
    let mut fifo = BgFifo::new();

    let mut rng = rand::rng();
    let low: u8 = rng.random();
    let high: u8 = rng.random();

    fifo.push_tile(low, high);

    for bit in (0..8).rev() {
        let low_bit = (low >> bit) & 1;
        let high_bit = (high >> bit) & 1;
        let expected = (high_bit << 1) | low_bit;

        assert_eq!(fifo.pop(), Some(expected));
    }

    assert!(fifo.is_empty());
}
