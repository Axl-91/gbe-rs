use std::collections::VecDeque;

pub struct BgFifo {
    pixels: VecDeque<u8>,
}

impl BgFifo {
    pub fn new() -> Self {
        Self {
            pixels: VecDeque::new(),
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.pixels.is_empty()
    }

    pub(super) fn push(&mut self, pixel: u8) {
        self.pixels.push_back(pixel);
    }

    pub(super) fn pop(&mut self) -> Option<u8> {
        self.pixels.pop_front()
    }

    pub(super) fn clear(&mut self) {
        self.pixels.clear();
    }

    pub(super) fn push_tile(&mut self, low: u8, high: u8) {
        for bit in (0..8).rev() {
            let low_bit = (low >> bit) & 1;
            let high_bit = (high >> bit) & 1;

            let pixel = (high_bit << 1) | low_bit;

            self.push(pixel);
        }
    }
}
