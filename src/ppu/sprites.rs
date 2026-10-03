//! Sprite handling for the Game Boy PPU.

use crate::ppu::Ppu;

const SPRITE_HEIGHT_8X8: u8 = 8;
const SPRITE_HEIGHT_8X16: u8 = 16;
const MAX_SPRITES_PER_LINE: usize = 10;

const OAM_ENTRY_SIZE: u16 = 4;
const SPRITE_Y_OFFSET: u8 = 16;

const OAM_SEARCH_CYCLES: u8 = 2;

const ATTRIBUTE_PRIORITY: u8 = 7;

/// A sprite selected during OAM Search.
#[derive(Clone, Copy, Default)]
pub(super) struct Sprite {
    pub x: u8,
    pub y: u8,
    pub tile: u8,
    pub attributes: u8,
}

impl Sprite {
    pub(super) fn get_x(&self) -> u8 {
        self.x
    }

    pub(super) fn has_priority(&self) -> bool {
        self.attributes & (1 << ATTRIBUTE_PRIORITY) != 0
    }
}

pub(super) struct OamSearcher {
    cycles: u8,
    index: u8,
}

impl OamSearcher {
    pub(super) fn new() -> Self {
        Self {
            cycles: 0,
            index: 0,
        }
    }

    pub(super) fn reset(&mut self) {
        self.cycles = 0;
        self.index = 0;
    }

    pub(super) fn tick(&mut self) -> Option<u8> {
        self.cycles += 1;

        if self.cycles < OAM_SEARCH_CYCLES {
            return None;
        }

        self.cycles = 0;

        let index = self.index;
        self.index += 1;

        Some(index)
    }
}

impl Ppu {
    pub(super) fn add_sprite(&mut self, index: u8) {
        if self.sprites.len() == MAX_SPRITES_PER_LINE {
            return;
        }

        let sprite_height = if self.is_obj_size() {
            SPRITE_HEIGHT_8X16
        } else {
            SPRITE_HEIGHT_8X8
        };
        let line = self.ly.wrapping_add(SPRITE_Y_OFFSET);

        let address = index as u16 * OAM_ENTRY_SIZE;
        let y = self.oam[address as usize];

        if line >= y && line < y.saturating_add(sprite_height) {
            let x = self.oam[address as usize + 1];
            let tile = self.oam[address as usize + 2];
            let attributes = self.oam[address as usize + 3];

            self.sprites.push(Sprite {
                y,
                x,
                tile,
                attributes,
            });
        }
    }
}
