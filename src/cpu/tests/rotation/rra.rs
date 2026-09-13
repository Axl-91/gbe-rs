use super::*;

#[test]
fn rra() {
    let mut rng = rand::rng();

    let a = rng.random_range(0..=u8::MAX);
    let carry = rng.random_bool(0.5);

    let mut cpu = create_cpu(0x1F, None, None);

    cpu.registers.set_a(a);
    cpu.registers.set_carry(carry);

    let new_carry = a & 0x01;
    let expected_a = (a >> 1) | ((carry as u8) << 7);

    cpu.step();

    assert_eq!(cpu.registers.get_a(), expected_a);
    assert!(!cpu.registers.get_zero());
    assert!(!cpu.registers.get_subtract());
    assert!(!cpu.registers.get_half_carry());
    assert_eq!(cpu.registers.get_carry(), new_carry != 0);
}
