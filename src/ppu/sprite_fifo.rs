// TODO: Once all functions are used we can delete this
#![allow(dead_code)]

use std::collections::VecDeque;

use crate::ppu::{SCREEN_WIDTH, Sprite};

const SPRITE_SIZE: i16 = 8;

#[derive(Clone, Copy)]
enum SpritePixel {
    Empty,
    Transparent,
    Color(u8),
}

pub(super) struct SpriteFifo {
    pixels: VecDeque<SpritePixel>,
}

impl SpriteFifo {
    pub(super) fn new() -> Self {
        Self {
            pixels: (0..SCREEN_WIDTH).map(|_| SpritePixel::Empty).collect(),
        }
    }

    fn len(&self) -> usize {
        self.pixels.len()
    }

    fn is_empty(&self) -> bool {
        self.pixels.is_empty()
    }

    fn pop(&mut self) -> Option<u8> {
        self.pixels.pop_front().map(|pixel| match pixel {
            SpritePixel::Empty | SpritePixel::Transparent => 0,
            SpritePixel::Color(color) => color,
        })
    }

    fn clear(&mut self) {
        self.pixels.clear();
    }

    fn should_push_sprite(&self, sprite: &Sprite) -> bool {
        let real_x_pos = sprite.x as i16 - SPRITE_SIZE;

        (0..SPRITE_SIZE).any(|pixel| {
            let pos_x = real_x_pos + pixel;

            if pos_x < 0 {
                return false;
            }
            matches!(self.pixels.get(pos_x as usize), Some(SpritePixel::Empty))
        })
    }

    pub(super) fn push_tile(&mut self, sprite: &Sprite, low: u8, high: u8) {
        let real_x_pos = sprite.x as i16 - SPRITE_SIZE;

        for pixel in 0..SPRITE_SIZE {
            let pos_x = real_x_pos + pixel;

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
                SpritePixel::Color(color)
            };

            let pos_x = pos_x as usize;

            if matches!(self.pixels[pos_x], SpritePixel::Empty) {
                self.pixels[pos_x] = sprite_pixel;
            }
        }
    }
}
