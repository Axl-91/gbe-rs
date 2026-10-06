use super::*;

// These tests use HRAM only because they verify the load instructions,
// not the specific behavior of I/O registers. Other high-memory addresses
// may have register-specific read/write behavior that belongs to separate tests.
const HRAM_OFFSET_RANGE: std::ops::RangeInclusive<u8> = 0x80..=0xFE;

#[test]
fn load8_to_high_address() {
    let mut rng = rand::rng();

    let a = rng.random();
    let offset = rng.random_range(HRAM_OFFSET_RANGE);

    let mut cpu = create_cpu(0xE0, Some(offset), None);

    cpu.registers.set_a(a);
    cpu.step();

    let high_address = 0xFF00 as u16;
    let address = high_address.wrapping_add(offset as u16);

    assert_eq!(cpu.bus.read(address), a);
}

#[test]
fn load8_from_high_address() {
    let mut rng = rand::rng();

    let value = rng.random();
    let offset = rng.random_range(HRAM_OFFSET_RANGE);

    let mut cpu = create_cpu(0xF0, Some(offset), None);

    let high_address = 0xFF00 as u16;
    let address = high_address.wrapping_add(offset as u16);

    cpu.bus.write(address, value);

    cpu.step();

    assert_eq!(cpu.registers.get_a(), value);
}

#[test]
fn load8_to_high_register() {
    let mut rng = rand::rng();

    let a = rng.random();
    let register_c = rng.random_range(HRAM_OFFSET_RANGE);

    let mut cpu = create_cpu(0xE2, None, None);

    cpu.registers.set_a(a);
    cpu.registers.set_c(register_c);

    cpu.step();

    let high_address = 0xFF00 as u16;
    let address = high_address.wrapping_add(register_c as u16);

    assert_eq!(cpu.bus.read(address), a);
}

#[test]
fn load8_from_high_register() {
    let mut rng = rand::rng();

    let value = rng.random();
    let register_c = rng.random_range(HRAM_OFFSET_RANGE);

    let mut cpu = create_cpu(0xF2, None, None);

    let high_address = 0xFF00 as u16;
    let address = high_address.wrapping_add(register_c as u16);

    cpu.bus.write(address, value);
    cpu.registers.set_c(register_c);

    cpu.step();

    assert_eq!(cpu.registers.get_a(), value);
}
