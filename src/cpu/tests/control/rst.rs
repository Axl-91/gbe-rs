use super::*;

#[test]
fn rst() {
    let vectors = [
        (0xC7, 0x00),
        (0xCF, 0x08),
        (0xD7, 0x10),
        (0xDF, 0x18),
        (0xE7, 0x20),
        (0xEF, 0x28),
        (0xF7, 0x30),
        (0xFF, 0x38),
    ];

    for (opcode, vector) in vectors {
        let mut cpu = create_cpu(opcode, None, None);

        let sp = get_rand_wram_address();
        cpu.registers.set_sp(sp);

        cpu.step();

        let expected_sp = sp.wrapping_sub(2);
        let return_address = GAME_ENTRY_POINT + 1;
        let [return_low, return_high] = return_address.to_le_bytes();

        assert_eq!(cpu.registers.get_pc(), vector);
        assert_eq!(cpu.registers.get_sp(), expected_sp);
        assert_eq!(cpu.bus.read(expected_sp), return_low);
        assert_eq!(cpu.bus.read(expected_sp + 1), return_high);
    }
}
