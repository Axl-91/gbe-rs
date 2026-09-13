use super::*;

#[test]
fn daa_addition_no_correction() {
    let mut rng = rand::rng();

    let high = rng.random_range(0..=9);
    let low = rng.random_range(0..=9);
    let a = (high << 4) | low;

    let mut cpu = create_cpu(0x27, None, None);

    cpu.registers.set_a(a);
    cpu.registers.set_subtract(false);
    cpu.registers.set_half_carry(false);
    cpu.registers.set_carry(false);

    cpu.step();

    assert_eq!(cpu.registers.get_a(), a);
    assert_eq!(cpu.registers.get_zero(), a == 0);
    assert!(!cpu.registers.get_subtract());
    assert!(!cpu.registers.get_half_carry());
    assert!(!cpu.registers.get_carry());
}

#[test]
fn daa_addition_low_correction() {
    let mut rng = rand::rng();

    let high = rng.random_range(0..=9);
    let low = rng.random_range(0xA..=0xF);
    let a = (high << 4) | low;

    let mut cpu = create_cpu(0x27, None, None);

    cpu.registers.set_a(a);
    cpu.registers.set_subtract(false);
    cpu.registers.set_half_carry(false);
    cpu.registers.set_carry(false);

    let expected = a.wrapping_add(0x06);

    cpu.step();

    assert_eq!(cpu.registers.get_a(), expected);
    assert_eq!(cpu.registers.get_zero(), expected == 0);
    assert!(!cpu.registers.get_subtract());
    assert!(!cpu.registers.get_half_carry());
    assert!(!cpu.registers.get_carry());
}

#[test]
fn daa_addition_high_correction() {
    let mut rng = rand::rng();

    let high = rng.random_range(0xA..=0xF);
    let low = rng.random_range(0..=9);
    let a = (high << 4) | low;

    let mut cpu = create_cpu(0x27, None, None);

    cpu.registers.set_a(a);
    cpu.registers.set_subtract(false);
    cpu.registers.set_half_carry(false);
    cpu.registers.set_carry(false);

    let expected = a.wrapping_add(0x60);

    cpu.step();

    assert_eq!(cpu.registers.get_a(), expected);
    assert_eq!(cpu.registers.get_zero(), expected == 0);
    assert!(!cpu.registers.get_subtract());
    assert!(!cpu.registers.get_half_carry());
    assert!(cpu.registers.get_carry());
}

#[test]
fn daa_addition_both_corrections() {
    let mut rng = rand::rng();

    let high = rng.random_range(0xA..=0xF);
    let low = rng.random_range(0xA..=0xF);
    let a = (high << 4) | low;

    let mut cpu = create_cpu(0x27, None, None);

    cpu.registers.set_a(a);
    cpu.registers.set_subtract(false);
    cpu.registers.set_half_carry(false);
    cpu.registers.set_carry(false);

    let expected = a.wrapping_add(0x06).wrapping_add(0x60);

    cpu.step();

    assert_eq!(cpu.registers.get_a(), expected);
    assert_eq!(cpu.registers.get_zero(), expected == 0);
    assert!(!cpu.registers.get_subtract());
    assert!(!cpu.registers.get_half_carry());
    assert!(cpu.registers.get_carry());
}

#[test]
fn daa_addition_half_carry() {
    let mut rng = rand::rng();

    let high = rng.random_range(0..=9);
    let low = rng.random_range(0..=9);
    let a = (high << 4) | low;

    let mut cpu = create_cpu(0x27, None, None);

    cpu.registers.set_a(a);
    cpu.registers.set_subtract(false);
    cpu.registers.set_half_carry(true);
    cpu.registers.set_carry(false);

    let expected = a.wrapping_add(0x06);

    cpu.step();

    assert_eq!(cpu.registers.get_a(), expected);
    assert_eq!(cpu.registers.get_zero(), expected == 0);
    assert!(!cpu.registers.get_subtract());
    assert!(!cpu.registers.get_half_carry());
    assert!(!cpu.registers.get_carry());
}

#[test]
fn daa_addition_carry() {
    let mut rng = rand::rng();

    let high = rng.random_range(0..=9);
    let low = rng.random_range(0..=9);
    let a = (high << 4) | low;

    let mut cpu = create_cpu(0x27, None, None);

    cpu.registers.set_a(a);
    cpu.registers.set_subtract(false);
    cpu.registers.set_half_carry(false);
    cpu.registers.set_carry(true);

    let expected = a.wrapping_add(0x60);

    cpu.step();

    assert_eq!(cpu.registers.get_a(), expected);
    assert_eq!(cpu.registers.get_zero(), expected == 0);
    assert!(!cpu.registers.get_subtract());
    assert!(!cpu.registers.get_half_carry());
    assert!(cpu.registers.get_carry());
}

#[test]
fn daa_subtraction_no_correction() {
    let mut rng = rand::rng();

    let a = rng.random_range(0..=u8::MAX);

    let mut cpu = create_cpu(0x27, None, None);

    cpu.registers.set_a(a);
    cpu.registers.set_subtract(true);
    cpu.registers.set_half_carry(false);
    cpu.registers.set_carry(false);

    cpu.step();

    assert_eq!(cpu.registers.get_a(), a);
    assert_eq!(cpu.registers.get_zero(), a == 0);
    assert!(cpu.registers.get_subtract());
    assert!(!cpu.registers.get_half_carry());
    assert!(!cpu.registers.get_carry());
}

#[test]
fn daa_subtraction_low_correction() {
    let mut rng = rand::rng();

    let a = rng.random_range(0..=u8::MAX);

    let mut cpu = create_cpu(0x27, None, None);

    cpu.registers.set_a(a);
    cpu.registers.set_subtract(true);
    cpu.registers.set_half_carry(true);
    cpu.registers.set_carry(false);

    let expected = a.wrapping_sub(0x06);

    cpu.step();

    assert_eq!(cpu.registers.get_a(), expected);
    assert_eq!(cpu.registers.get_zero(), expected == 0);
    assert!(cpu.registers.get_subtract());
    assert!(!cpu.registers.get_half_carry());
    assert!(!cpu.registers.get_carry());
}

#[test]
fn daa_subtraction_high_correction() {
    let mut rng = rand::rng();

    let a = rng.random_range(0..=u8::MAX);

    let mut cpu = create_cpu(0x27, None, None);

    cpu.registers.set_a(a);
    cpu.registers.set_subtract(true);
    cpu.registers.set_half_carry(false);
    cpu.registers.set_carry(true);

    let expected = a.wrapping_sub(0x60);

    cpu.step();

    assert_eq!(cpu.registers.get_a(), expected);
    assert_eq!(cpu.registers.get_zero(), expected == 0);
    assert!(cpu.registers.get_subtract());
    assert!(!cpu.registers.get_half_carry());
    assert!(cpu.registers.get_carry());
}

#[test]
fn daa_subtraction_both_corrections() {
    let mut rng = rand::rng();

    let a = rng.random_range(0..=u8::MAX);

    let mut cpu = create_cpu(0x27, None, None);

    cpu.registers.set_a(a);
    cpu.registers.set_subtract(true);
    cpu.registers.set_half_carry(true);
    cpu.registers.set_carry(true);

    let expected = a.wrapping_sub(0x06).wrapping_sub(0x60);

    cpu.step();

    assert_eq!(cpu.registers.get_a(), expected);
    assert_eq!(cpu.registers.get_zero(), expected == 0);
    assert!(cpu.registers.get_subtract());
    assert!(!cpu.registers.get_half_carry());
    assert!(cpu.registers.get_carry());
}
