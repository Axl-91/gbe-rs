use super::*;

use rand::RngExt;

const PIXELS_PER_TILE: usize = 8;
const MAX_TILES: usize = 8;

const STARTUP_DELAY: u8 = 6;

fn complete_start_up_phase(fetcher: &mut Fetcher) {
    for _ in 0..STARTUP_DELAY {
        fetcher.tick();
    }
}

mod fetcher;
mod fifo;
mod memory;
mod registers;
mod windows;
