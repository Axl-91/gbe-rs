use super::*;

use rand::RngExt;

const PIXELS_PER_TILE: usize = 8;
const MAX_TILES: usize = 8;

mod fetcher;
mod fifo;
mod memory;
mod registers;
