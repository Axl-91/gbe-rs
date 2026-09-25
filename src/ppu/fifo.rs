// TODO: Once all functions are used we can delete this
#![allow(dead_code)]

use std::collections::VecDeque;

const PIXELS_PER_TILE: usize = 8;
const MAX_TILES: usize = 8;

pub struct PixelFifo {
    pixels: VecDeque<u8>,
}

impl PixelFifo {
    pub fn new() -> Self {
        Self {
            pixels: VecDeque::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.pixels.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pixels.is_empty()
    }

    pub fn push(&mut self, pixel: u8) {
        self.pixels.push_back(pixel);
    }

    pub fn pop(&mut self) -> Option<u8> {
        self.pixels.pop_front()
    }

    pub fn can_push_tile(&self) -> bool {
        let new_amount_pixels = self.pixels.len() + PIXELS_PER_TILE;
        let max_pixels = MAX_TILES * PIXELS_PER_TILE;

        new_amount_pixels <= max_pixels
    }

    pub fn push_tile(&mut self, low: u8, high: u8) {
        for bit in (0..8).rev() {
            let low_bit = (low >> bit) & 1;
            let high_bit = (high >> bit) & 1;

            let pixel = (high_bit << 1) | low_bit;

            self.push(pixel);
        }
    }
}
