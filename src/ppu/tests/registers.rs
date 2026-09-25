use rand::RngExt;

use crate::ppu::{
    Ppu,
    memory::{LCDC_ADDRESS, LY_ADDRESS, LYC_ADDRESS, STAT_ADDRESS},
};

#[test]
fn lcdc_write_disables_lcd_and_resets_ly() {
    let mut ppu = Ppu::new();

    ppu.write(LCDC_ADDRESS, 0);

    assert_eq!(ppu.read(LY_ADDRESS), 0);
}

#[test]
fn lcdc_write_enables_lcd() {
    let mut ppu = Ppu::new();

    ppu.write(LCDC_ADDRESS, 0);
    ppu.write(LCDC_ADDRESS, 1);

    assert_ne!(ppu.read(LCDC_ADDRESS), 0);
}

#[test]
fn ly_is_read_only() {
    let mut ppu = Ppu::new();

    let mut rng = rand::rng();
    let value: u8 = rng.random();

    ppu.write(LY_ADDRESS, value);

    assert_eq!(ppu.read(LY_ADDRESS), 0);
}

#[test]
fn lyc_can_be_written_and_read() {
    let mut ppu = Ppu::new();

    let mut rng = rand::rng();
    let value: u8 = rng.random();

    ppu.write(LYC_ADDRESS, value);

    assert_eq!(ppu.read(LYC_ADDRESS), value);
}

#[test]
fn stat_write_cannot_modify_mode_bits() {
    let mut ppu = Ppu::new();

    ppu.write(LCDC_ADDRESS, 0b1000_0000);

    let mode_before = ppu.read_stat() & 0b11;

    let mut rng = rand::rng();
    let value: u8 = rng.random();

    ppu.write(STAT_ADDRESS, value);

    let mode_after = ppu.read_stat() & 0b11;

    assert_eq!(mode_after, mode_before);
}
