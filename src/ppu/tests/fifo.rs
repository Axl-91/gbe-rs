use super::*;

#[test]
fn new_fifo_is_empty() {
    let fifo = PixelFifo::new();

    assert!(fifo.is_empty());
    assert_eq!(fifo.len(), 0);
}

#[test]
fn push_and_pop_preserve_order() {
    let mut fifo = PixelFifo::new();

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
    let mut fifo = PixelFifo::new();

    assert_eq!(fifo.pop(), None);
}

#[test]
fn push_tile_adds_eight_pixels() {
    let mut fifo = PixelFifo::new();

    let mut rng = rand::rng();
    let low: u8 = rng.random();
    let high: u8 = rng.random();

    fifo.push_tile(low, high);

    assert_eq!(fifo.len(), PIXELS_PER_TILE);
}

#[test]
fn push_tile_decodes_pixels_in_order() {
    let mut fifo = PixelFifo::new();

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

#[test]
fn cannot_push_tile_when_fifo_is_full() {
    let mut fifo = PixelFifo::new();

    let mut rng = rand::rng();

    while fifo.can_push_tile() {
        let low: u8 = rng.random();
        let high: u8 = rng.random();

        fifo.push_tile(low, high);
    }

    assert_eq!(fifo.len(), MAX_TILES * PIXELS_PER_TILE);
    assert!(!fifo.can_push_tile());
}

#[test]
fn can_push_tile_after_consuming_pixels() {
    let mut fifo = PixelFifo::new();

    let mut rng = rand::rng();

    while fifo.can_push_tile() {
        fifo.push(rng.random_range(0..=3));
    }

    assert!(!fifo.can_push_tile());

    fifo.pop();

    assert!(fifo.can_push_tile());
}
