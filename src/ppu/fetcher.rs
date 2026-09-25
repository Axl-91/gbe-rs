// use crate::{memory::map::VRAM_START, ppu::Ppu};

// const TILE_DATA_SIGNED_START: u16 = 0x9000;
// const TILE_SIZE_BYTES: u16 = 16;

// impl Ppu {
//     fn tile_data_address(&self, tile_index: u8) -> u16 {
//         if self.is_bg_window_tile_data() {
//             let offset = tile_index as u16 * TILE_SIZE_BYTES;
//             VRAM_START + offset
//         } else {
//             let signed_index = tile_index as i8 as i16;
//             let offset = signed_index * TILE_SIZE_BYTES as i16;

//             (TILE_DATA_SIGNED_START as i16 + offset) as u16
//         }
//     }
// }
