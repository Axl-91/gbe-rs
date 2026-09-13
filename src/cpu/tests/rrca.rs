use super::*;

#[test]
fn rrca() {
    let mut rng = rand::rng();

    let a = rng.random_range(0..=u8::MAX);

    let mut cpu = create_cpu(0x0F, None, None);

    cpu.registers.set_a(a);

    let new_carry = a & 0x01;
    let expected_a = (a >> 1) | (a << 7);

    cpu.step();

    assert_eq!(cpu.registers.get_a(), expected_a);
    assert!(!cpu.registers.get_zero());
    assert!(!cpu.registers.get_subtract());
    assert!(!cpu.registers.get_half_carry());
    assert_eq!(cpu.registers.get_carry(), new_carry != 0);
}
