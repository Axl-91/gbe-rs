use super::*;

#[test]
fn fetch_reads_opcode_and_advances_pc() {
    let mut rng = rand::rng();

    let mut rom = vec![0; ROM_BANK_SIZE * 2];

    let opcode: u8 = rng.random();
    rom[GAME_ENTRY_POINT as usize] = opcode;

    let cartridge = Cartridge::new(rom, 0);
    let bus = MemoryBus::new(cartridge);
    let mut cpu = Cpu::new(bus);

    let fetched = cpu.fetch();

    assert_eq!(fetched, opcode);
    assert_eq!(cpu.registers.get_pc(), GAME_ENTRY_POINT + 1);
}
