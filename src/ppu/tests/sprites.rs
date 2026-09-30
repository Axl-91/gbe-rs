use super::*;
use rand::RngExt;

const SPRITE_Y_OFFSET: u8 = 16;
const SPRITE_HEIGHT_8X8: u8 = 8;
const MAX_SPRITES_PER_LINE: usize = 10;
const OAM_ENTRY_SIZE: u16 = 4;
const OAM_ENTRY_COUNT: u8 = 40;

fn write_sprite(ppu: &mut Ppu, index: usize, sprite: Sprite) {
    let address = index * OAM_ENTRY_SIZE as usize;

    ppu.oam[address] = sprite.y;
    ppu.oam[address + 1] = sprite.x;
    ppu.oam[address + 2] = sprite.tile;
    ppu.oam[address + 3] = sprite.attributes;
}

fn random_sprite_y_for_line(ly: u8, height: u8) -> u8 {
    let mut rng = rand::rng();
    let line = ly + SPRITE_Y_OFFSET;

    line - rng.random_range(0..height)
}

#[test]
fn oam_search_returns_one_index_every_two_cycles() {
    let mut searcher = OamSearcher::new();

    assert_eq!(searcher.tick(), None);
    assert_eq!(searcher.tick(), Some(0));

    assert_eq!(searcher.tick(), None);
    assert_eq!(searcher.tick(), Some(1));

    assert_eq!(searcher.tick(), None);
    assert_eq!(searcher.tick(), Some(2));
}

#[test]
fn oam_search_processes_all_oam_entries() {
    let mut searcher = OamSearcher::new();

    for expected_index in 0..OAM_ENTRY_COUNT {
        assert_eq!(searcher.tick(), None);
        assert_eq!(searcher.tick(), Some(expected_index));
    }

    assert_eq!(searcher.tick(), None);
    assert_eq!(searcher.tick(), None);
}

#[test]
fn oam_search_reset_starts_from_first_entry() {
    let mut searcher = OamSearcher::new();

    searcher.tick();
    searcher.tick();

    searcher.reset();

    assert_eq!(searcher.tick(), None);
    assert_eq!(searcher.tick(), Some(0));
}

#[test]
fn sprite_visible_on_current_line() {
    let mut ppu = Ppu::new();
    let mut rng = rand::rng();

    let ly = rng.random_range(0..144);
    ppu.ly = ly;

    let sprite = Sprite {
        x: rng.random(),
        y: random_sprite_y_for_line(ly, SPRITE_HEIGHT_8X8),
        tile: rng.random(),
        attributes: rng.random(),
    };

    write_sprite(&mut ppu, 0, sprite);

    ppu.add_sprite(0);

    assert_eq!(ppu.sprites.len(), 1);
    assert_eq!(ppu.sprites[0].x, sprite.x);
    assert_eq!(ppu.sprites[0].y, sprite.y);
    assert_eq!(ppu.sprites[0].tile, sprite.tile);
    assert_eq!(ppu.sprites[0].attributes, sprite.attributes);
}

#[test]
fn sprite_above_current_line_is_ignored() {
    let mut ppu = Ppu::new();
    let mut rng = rand::rng();

    let ly = rng.random_range(0..144);
    ppu.ly = ly;

    let line = ly + SPRITE_Y_OFFSET;
    let y = line.saturating_sub(SPRITE_HEIGHT_8X8);

    let sprite = Sprite {
        x: rng.random(),
        y,
        tile: rng.random(),
        attributes: rng.random(),
    };

    write_sprite(&mut ppu, 0, sprite);

    ppu.add_sprite(0);

    assert!(ppu.sprites.is_empty());
}

#[test]
fn sprite_below_current_line_is_ignored() {
    let mut ppu = Ppu::new();
    let mut rng = rand::rng();

    let ly = rng.random_range(0..144);
    ppu.ly = ly;

    let line = ly + SPRITE_Y_OFFSET;
    let y = line + SPRITE_HEIGHT_8X8;

    let sprite = Sprite {
        x: rng.random(),
        y,
        tile: rng.random(),
        attributes: rng.random(),
    };

    write_sprite(&mut ppu, 0, sprite);

    ppu.add_sprite(0);

    assert!(ppu.sprites.is_empty());
}

#[test]
fn sprite_height_8x16_is_respected() {
    let mut ppu = Ppu::new();
    let mut rng = rand::rng();

    ppu.lcdc |= 1 << 2;

    let ly = rng.random_range(0..144);
    ppu.ly = ly;

    let line = ly + SPRITE_Y_OFFSET;
    let y = line - SPRITE_HEIGHT_8X8;

    let sprite = Sprite {
        x: rng.random(),
        y,
        tile: rng.random(),
        attributes: rng.random(),
    };

    write_sprite(&mut ppu, 0, sprite);

    ppu.add_sprite(0);

    assert_eq!(ppu.sprites.len(), 1);
}

#[test]
fn only_ten_sprites_are_selected() {
    let mut ppu = Ppu::new();
    let mut rng = rand::rng();

    let ly = rng.random_range(0..144);
    ppu.ly = ly;

    for index in 0..OAM_ENTRY_COUNT as usize {
        let sprite = Sprite {
            x: rng.random(),
            y: random_sprite_y_for_line(ly, SPRITE_HEIGHT_8X8),
            tile: rng.random(),
            attributes: rng.random(),
        };

        write_sprite(&mut ppu, index, sprite);
    }

    for index in 0..OAM_ENTRY_COUNT {
        ppu.add_sprite(index);
    }

    assert_eq!(ppu.sprites.len(), MAX_SPRITES_PER_LINE);
}

#[test]
fn oam_search_stops_after_all_entries() {
    let mut searcher = OamSearcher::new();

    for _ in 0..OAM_ENTRY_COUNT {
        searcher.tick();
        searcher.tick();
    }

    for _ in 0..OAM_ENTRY_COUNT {
        assert_eq!(searcher.tick(), None);
    }
}

#[test]
fn sprites_after_ten_are_ignored() {
    let mut ppu = Ppu::new();
    let mut rng = rand::rng();

    let ly = rng.random_range(0..144);
    ppu.ly = ly;

    for index in 0..MAX_SPRITES_PER_LINE {
        let sprite = Sprite {
            x: rng.random(),
            y: random_sprite_y_for_line(ly, SPRITE_HEIGHT_8X8),
            tile: rng.random(),
            attributes: rng.random(),
        };

        write_sprite(&mut ppu, index, sprite);
        ppu.add_sprite(index as u8);
    }

    let extra_sprite = Sprite {
        x: rng.random(),
        y: random_sprite_y_for_line(ly, SPRITE_HEIGHT_8X8),
        tile: rng.random(),
        attributes: rng.random(),
    };

    write_sprite(&mut ppu, MAX_SPRITES_PER_LINE, extra_sprite);
    ppu.add_sprite(MAX_SPRITES_PER_LINE as u8);

    assert_eq!(ppu.sprites.len(), MAX_SPRITES_PER_LINE);
}

#[test]
fn oam_search_reset_clears_current_cycle() {
    let mut searcher = OamSearcher::new();

    searcher.tick();

    searcher.reset();

    assert_eq!(searcher.tick(), None);
    assert_eq!(searcher.tick(), Some(0));
}
