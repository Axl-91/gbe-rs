use rand::RngExt;

use crate::cartridge::Cartridge;
use crate::memory::MemoryBus;
use crate::memory::map::ROM_BANK_SIZE;

use super::super::*;

fn create_cpu(opcode: u8) -> Cpu {
    let mut rom = vec![0; ROM_BANK_SIZE * 2];

    rom[GAME_ENTRY_POINT as usize] = opcode;

    let cartridge = Cartridge::new(rom, 0);
    let bus = MemoryBus::new(cartridge);

    Cpu::new(bus)
}

#[test]
fn inc_increments_all_registers() {
    let mut rng = rand::rng();

    let registers = [
        (0x04, Register8::B),
        (0x0C, Register8::C),
        (0x14, Register8::D),
        (0x1C, Register8::E),
        (0x24, Register8::H),
        (0x2C, Register8::L),
        (0x3C, Register8::A),
    ];

    for (opcode, register) in registers {
        let value: u8 = rng.random_range(0..=0xFE);

        let mut cpu = create_cpu(opcode);

        cpu.set_register8(&register, value);

        cpu.step();

        assert_eq!(cpu.get_register8(&register), value + 1);
    }
}

#[test]
fn inc_sets_zero_flag_when_result_is_zero() {
    let mut cpu = create_cpu(0x04);

    cpu.registers.set_b(0xFF);

    cpu.step();

    assert_eq!(cpu.registers.get_b(), 0x00);
    assert!(cpu.registers.get_zero());
}

#[test]
fn inc_clears_zero_flag_when_result_is_not_zero() {
    let mut rng = rand::rng();
    let value: u8 = rng.random_range(0..=0xFE);

    let mut cpu = create_cpu(0x04);

    cpu.registers.set_b(value);
    cpu.registers.set_zero(true);

    cpu.step();

    assert!(!cpu.registers.get_zero());
}

#[test]
fn inc_clears_subtract_flag() {
    let mut cpu = create_cpu(0x04);

    cpu.registers.set_b(0x05);
    cpu.registers.set_subtract(true);

    cpu.step();

    assert!(!cpu.registers.get_subtract());
}

#[test]
fn inc_sets_half_carry_when_lower_nibble_overflows() {
    let mut cpu = create_cpu(0x04);

    cpu.registers.set_b(0x0F);

    cpu.step();

    assert_eq!(cpu.registers.get_b(), 0x10);
    assert!(cpu.registers.get_half_carry());
}

#[test]
fn inc_clears_half_carry_when_lower_nibble_does_not_overflow() {
    let mut rng = rand::rng();
    let value: u8 = rng.random_range(0..=0x0E);

    let mut cpu = create_cpu(0x04);

    cpu.registers.set_b(value);
    cpu.registers.set_half_carry(true);

    cpu.step();

    assert!(!cpu.registers.get_half_carry());
}

#[test]
fn inc_preserves_carry_flag() {
    let mut rng = rand::rng();
    let value: u8 = rng.random();

    let mut cpu = create_cpu(0x04);

    cpu.registers.set_b(value);
    cpu.registers.set_carry(true);

    cpu.step();

    assert!(cpu.registers.get_carry());
}
