use rand::RngExt;

use crate::cartridge::Cartridge;
use crate::cartridge::mbc::CartridgeMbc;
use crate::cartridge::mbc1::Mbc1;
use crate::cpu::interrupt::Interruption;
use crate::memory::MemoryBus;
use crate::memory::map::{INTERRUPT_ENABLE_ADDRESS, INTERRUPT_FLAG_ADDRESS, ROM_BANK_SIZE};

use super::*;

#[test]
fn reti_restores_pc_and_enables_interrupts() {
    let mut rng = rand::rng();

    let return_pc = rng.random();
    let initial_sp = get_rand_wram_address();

    let mut cpu = create_cpu(0xD9, None, None);

    cpu.registers.set_sp(initial_sp);
    cpu.ime = false;

    cpu.push_into_sp(return_pc);

    let t_cycles = cpu.step();

    assert_eq!(cpu.registers.get_pc(), return_pc);
    assert_eq!(cpu.registers.get_sp(), initial_sp);
    assert!(cpu.ime);
    assert_eq!(t_cycles, 16);
}

#[test]
fn interruption_followed_by_reti_restores_cpu_state() {
    let mut rng = rand::rng();

    let return_pc = rng.random_range(0x0100..=0x7FFF);
    let initial_sp = get_rand_wram_address();

    let mut rom = vec![0; ROM_BANK_SIZE * 2];

    rom[0x0040] = 0xD9;

    let mbc = CartridgeMbc::Mbc1(Mbc1::new());
    let cartridge = Cartridge::new(rom, 0, mbc);
    let bus = MemoryBus::new(cartridge);
    let mut cpu = Cpu::new(bus);

    cpu.registers.set_pc(return_pc);
    cpu.registers.set_sp(initial_sp);
    cpu.ime = true;

    let interrupt_bit = Interruption::VBlank.bit();
    let interrupt_mask = 0x01 << interrupt_bit;

    cpu.bus.write(INTERRUPT_ENABLE_ADDRESS, interrupt_mask);
    cpu.bus.write(INTERRUPT_FLAG_ADDRESS, interrupt_mask);

    let interruption_cycles = cpu.step();

    assert_eq!(interruption_cycles, 20);
    assert_eq!(cpu.registers.get_pc(), Interruption::VBlank.vector());
    assert!(!cpu.ime);
    assert_eq!(cpu.bus.read(INTERRUPT_FLAG_ADDRESS) & interrupt_mask, 0);

    let sp_after_interruption = cpu.registers.get_sp();

    assert_eq!(sp_after_interruption + 2, initial_sp);

    let reti_cycles = cpu.step();

    assert_eq!(reti_cycles, 16);
    assert_eq!(cpu.registers.get_pc(), return_pc);
    assert_eq!(cpu.registers.get_sp(), initial_sp);
    assert!(cpu.ime);
}
