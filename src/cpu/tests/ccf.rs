use super::*;

#[test]
fn ccf() {
    let mut rng = rand::rng();

    let zero = rng.random_bool(0.5);
    let subtract = rng.random_bool(0.5);
    let half_carry = rng.random_bool(0.5);
    let carry = rng.random_bool(0.5);

    let mut cpu = create_cpu(0x3F, None, None);

    cpu.registers.set_zero(zero);
    cpu.registers.set_subtract(subtract);
    cpu.registers.set_half_carry(half_carry);
    cpu.registers.set_carry(carry);

    cpu.step();

    assert_eq!(cpu.registers.get_zero(), zero);
    assert!(!cpu.registers.get_subtract());
    assert!(!cpu.registers.get_half_carry());
    assert_eq!(cpu.registers.get_carry(), !carry);
}
