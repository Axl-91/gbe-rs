use std::collections::VecDeque;

use crate::ppu::{SCREEN_WIDTH, SPRITE_SIZE, sprites::SpriteAttributes};

#[derive(Clone, Copy)]
pub enum SpritePixel {
    Empty,
    Transparent,
    Color {
        color: u8,
        priority: bool,
        dmg_palette: bool,
    },
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

    pub(super) fn uses_obp1(&self) -> bool {
        match self {
            SpritePixel::Color { dmg_palette, .. } => *dmg_palette,
            _ => false,
        }
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

    pub fn reset(&mut self) {
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

    pub(super) fn push_tile(
        &mut self,
        sprite_pos: i16,
        sprite_attrs: SpriteAttributes,
        low: u8,
        high: u8,
    ) {
        for pixel in 0..SPRITE_SIZE {
            let pos_x = sprite_pos + pixel;

            if pos_x < 0 {
                continue;
            }

            let bit = if sprite_attrs.x_flip {
                pixel
            } else {
                (SPRITE_SIZE - 1) - pixel
            };

            let low_bit = (low >> bit) & 1;
            let high_bit = (high >> bit) & 1;

            let color = (high_bit << 1) | low_bit;

            let sprite_pixel = if color == 0 {
                SpritePixel::Transparent
            } else {
                let priority = sprite_attrs.obj_to_bg_priority;
                let dmg_palette = sprite_attrs.dmg_palette;

                SpritePixel::Color {
                    color,
                    priority,
                    dmg_palette,
                }
            };

            let pos_x = pos_x as usize;

            // If the pixel is outside the visible area, we don't take it into account.
            if let Some(pixel) = self.pixels.get_mut(pos_x) {
                if matches!(pixel, SpritePixel::Empty) {
                    *pixel = sprite_pixel;
                }
            }
        }
    }
}
