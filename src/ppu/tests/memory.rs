use rand::RngExt;

use crate::{
    memory::map::{OAM_END, OAM_START, VRAM_END, VRAM_START},
    ppu::{
        Ppu, PpuMode,
        memory::{
            BGP_ADDRESS, LCDC_ADDRESS, LYC_ADDRESS, OBP0_ADDRESS, OBP1_ADDRESS, SCX_ADDRESS,
            SCY_ADDRESS, STAT_ADDRESS, WX_ADDRESS, WY_ADDRESS,
        },
    },
};

#[test]
fn vram_can_be_read_and_written_when_lcd_is_disabled() {
    let mut ppu = Ppu::new();

    ppu.write(LCDC_ADDRESS, 0);

    let mut rng = rand::rng();
    let offset = rng.random_range(0..=(VRAM_END - VRAM_START));
    let address = VRAM_START + offset;
    let value: u8 = rng.random();

    ppu.write(address, value);

    assert_eq!(ppu.read(address), value);
}

#[test]
fn vram_is_blocked_during_drawing() {
    let mut ppu = Ppu::new();

    ppu.write(LCDC_ADDRESS, 0);

    let mut rng = rand::rng();
    let address = rng.random_range(VRAM_START..=VRAM_END);
    let value: u8 = rng.random();

    ppu.write(address, value);

    let lcdc = ppu.read(LCDC_ADDRESS);
    ppu.write(LCDC_ADDRESS, lcdc | (1 << 7));

    // Enter Drawing mode.
    while ppu.read_stat() & 0b11 != PpuMode::Drawing as u8 {
        ppu.tick();
    }

    assert_eq!(ppu.read(address), u8::MAX);
}

#[test]
fn oam_can_be_read_and_written_when_lcd_is_disabled() {
    let mut ppu = Ppu::new();

    ppu.write(LCDC_ADDRESS, 0);

    let mut rng = rand::rng();
    let offset = rng.random_range(0..=(OAM_END - OAM_START));
    let address = OAM_START + offset;
    let value: u8 = rng.random();

    ppu.write(address, value);

    assert_eq!(ppu.read(address), value);
}

#[test]
fn oam_is_blocked_during_drawing() {
    let mut ppu = Ppu::new();

    ppu.write(LCDC_ADDRESS, 0);

    let mut rng = rand::rng();
    let address = rng.random_range(OAM_START..=OAM_END);
    let value: u8 = rng.random();

    ppu.write(address, value);

    let lcdc = ppu.read(LCDC_ADDRESS);
    ppu.write(LCDC_ADDRESS, lcdc | (1 << 7));

    while ppu.read_stat() & 0b11 != PpuMode::Drawing as u8 {
        ppu.tick();
    }

    assert_eq!(ppu.read(address), u8::MAX);
}

#[test]
fn register_writes_are_read_back() {
    let mut ppu = Ppu::new();
    let mut rng = rand::rng();

    let scy: u8 = rng.random();
    let scx: u8 = rng.random();
    let lyc: u8 = rng.random();
    let bgp: u8 = rng.random();
    let obp0: u8 = rng.random();
    let obp1: u8 = rng.random();
    let wy: u8 = rng.random();
    let wx: u8 = rng.random();

    ppu.write(SCY_ADDRESS, scy);
    ppu.write(SCX_ADDRESS, scx);
    ppu.write(LYC_ADDRESS, lyc);
    ppu.write(BGP_ADDRESS, bgp);
    ppu.write(OBP0_ADDRESS, obp0);
    ppu.write(OBP1_ADDRESS, obp1);
    ppu.write(WY_ADDRESS, wy);
    ppu.write(WX_ADDRESS, wx);

    assert_eq!(ppu.read(SCY_ADDRESS), scy);
    assert_eq!(ppu.read(SCX_ADDRESS), scx);
    assert_eq!(ppu.read(LYC_ADDRESS), lyc);
    assert_eq!(ppu.read(BGP_ADDRESS), bgp);
    assert_eq!(ppu.read(OBP0_ADDRESS), obp0);
    assert_eq!(ppu.read(OBP1_ADDRESS), obp1);
    assert_eq!(ppu.read(WY_ADDRESS), wy);
    assert_eq!(ppu.read(WX_ADDRESS), wx);
}

#[test]
fn stat_write_only_updates_interrupt_enable_bits() {
    let mut ppu = Ppu::new();
    let mut rng = rand::rng();

    let value: u8 = rng.random();

    ppu.write(STAT_ADDRESS, value);

    assert_eq!(ppu.read_stat() & 0b0111_1000, value & 0b0111_1000);
}
