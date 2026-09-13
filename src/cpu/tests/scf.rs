use super::*;

#[test]
fn cpl() {
    let mut rng = rand::rng();

    let a = rng.random_range(0..=u8::MAX);
    let zero = rng.random_bool(0.5);
    let carry = rng.random_bool(0.5);

    let mut cpu = create_cpu(0x2F, None, None);

    cpu.registers.set_a(a);
    cpu.registers.set_zero(zero);
    cpu.registers.set_carry(carry);

    let expected = !a;

    cpu.step();

    assert_eq!(cpu.registers.get_a(), expected);
    assert_eq!(cpu.registers.get_zero(), zero);
    assert!(cpu.registers.get_subtract());
    assert!(cpu.registers.get_half_carry());
    assert_eq!(cpu.registers.get_carry(), carry);
}
