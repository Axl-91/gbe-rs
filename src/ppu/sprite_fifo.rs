// TODO: Once all functions are used we can delete this
#![allow(dead_code)]

use std::collections::VecDeque;

use crate::ppu::{SCREEN_WIDTH, SPRITE_SIZE};

#[derive(Clone, Copy)]
pub enum SpritePixel {
    Empty,
    Transparent,
    Color { color: u8, priority: bool },
}

impl SpritePixel {
    pub(super) fn get_color(&self) -> u8 {
        match self {
            SpritePixel::Empty | SpritePixel::Transparent => 0,
            SpritePixel::Color { color, .. } => *color,
        }
    }

    pub(super) fn is_behind_bg(&self) -> bool {
        matches!(self, SpritePixel::Color { priority: true, .. })
    }
}

pub(super) struct SpriteFifo {
    pixels: VecDeque<SpritePixel>,
}

fn create_pixels_queue() -> VecDeque<SpritePixel> {
    (0..SCREEN_WIDTH).map(|_| SpritePixel::Empty).collect()
}

impl SpriteFifo {
    pub(super) fn new() -> Self {
        Self {
            pixels: create_pixels_queue(),
        }
    }

    pub(super) fn pop(&mut self) -> Option<SpritePixel> {
        self.pixels.pop_front()
    }

    fn clear(&mut self) {
        self.pixels = create_pixels_queue();
    }

    pub(super) fn should_push_sprite(&self, sprite_pos: i16) -> bool {
        (0..SPRITE_SIZE).any(|pixel| {
            let pos_x = sprite_pos + pixel;

            if pos_x < 0 {
                return false;
            }

            matches!(self.pixels.get(pos_x as usize), Some(SpritePixel::Empty))
        })
    }

    pub(super) fn push_tile(&mut self, sprite_pos: i16, priority: bool, low: u8, high: u8) {
        for pixel in 0..SPRITE_SIZE {
            let pos_x = sprite_pos + pixel;

            if pos_x < 0 {
                continue;
            }

            let bit = (SPRITE_SIZE - 1) - pixel;
            let low_bit = (low >> bit) & 1;
            let high_bit = (high >> bit) & 1;

            let color = (high_bit << 1) | low_bit;

            let sprite_pixel = if color == 0 {
                SpritePixel::Transparent
            } else {
                SpritePixel::Color { color, priority }
            };

            let pos_x = pos_x as usize;

            if matches!(self.pixels[pos_x], SpritePixel::Empty) {
                self.pixels[pos_x] = sprite_pixel;
            }
        }
    }
}
