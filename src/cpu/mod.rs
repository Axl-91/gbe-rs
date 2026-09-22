//! Game Boy CPU implementation.
//!
//! Handles instruction fetching, decoding, execution, and CPU registers.

mod arithmetic;
mod cb;
mod control;
mod instruction;
mod interrupt;
mod load;
mod registers;
mod rotation;
mod stack;
mod t_cycles;

#[cfg(test)]
mod tests;

use crate::{cpu::instruction::*, memory::MemoryBus};
use log::info;
use registers::Registers;

const NON_CYCLES: u8 = 0x00;
const HALT_CYCLES: u8 = 0x04;
const TICK_CYCLES: u8 = 0x04;

/// Represents the Game Boy CPU and its connection to the memory bus.
pub struct Cpu {
    registers: Registers,
    bus: MemoryBus,
    ime: bool,
    ime_schedule: bool,
    halted: bool,
    halt_bug: bool,
    stopped: bool,
    total_t_cycles: u64,
}

impl Cpu {
    /// Creates a new CPU with initialized registers and the program counter
    /// set to the game's entry point.
    pub fn new(bus: MemoryBus) -> Self {
        let registers = Registers::new();

        Self {
            registers,
            bus,
            ime: false,
            ime_schedule: false,
            halted: false,
            halt_bug: false,
            stopped: false,
            total_t_cycles: 0,
        }
    }

    fn tick_read(&mut self, address: u16) -> u8 {
        self.total_t_cycles = self.total_t_cycles.wrapping_add(TICK_CYCLES as u64);

        let value = self.bus.read(address);

        self.bus.tick(TICK_CYCLES);
        value
    }

    fn tick_write(&mut self, address: u16, value: u8) {
        self.total_t_cycles = self.total_t_cycles.wrapping_add(TICK_CYCLES as u64);

        self.bus.write(address, value);
        self.bus.tick(TICK_CYCLES);
    }

    fn tick_internal(&mut self, m_cycles: u8) {
        let amount_t_cycles = TICK_CYCLES * m_cycles;
        self.total_t_cycles = self.total_t_cycles.wrapping_add(amount_t_cycles as u64);

        self.bus.tick(amount_t_cycles);
    }

    pub fn get_total_ticks(&self) -> u64 {
        self.total_t_cycles
    }

    /// Fetches the next opcode from memory and advances the program counter.
    fn fetch(&mut self) -> u8 {
        let pc = self.registers.get_pc();
        let opcode = self.tick_read(pc);

        if self.halt_bug {
            self.halt_bug = false;
        } else {
            let next_pc = pc.wrapping_add(1);
            self.registers.set_pc(next_pc);
        }
        opcode
    }

    fn get_register8(&self, register: &Register8) -> u8 {
        match register {
            Register8::A => self.registers.get_a(),
            Register8::B => self.registers.get_b(),
            Register8::C => self.registers.get_c(),
            Register8::D => self.registers.get_d(),
            Register8::E => self.registers.get_e(),
            Register8::H => self.registers.get_h(),
            Register8::L => self.registers.get_l(),
        }
    }

    fn set_register8(&mut self, register: &Register8, value: u8) {
        match register {
            Register8::A => self.registers.set_a(value),
            Register8::B => self.registers.set_b(value),
            Register8::C => self.registers.set_c(value),
            Register8::D => self.registers.set_d(value),
            Register8::E => self.registers.set_e(value),
            Register8::H => self.registers.set_h(value),
            Register8::L => self.registers.set_l(value),
        }
    }

    fn get_register16(&self, register: &Register16) -> u16 {
        match register {
            Register16::BC => self.registers.get_bc(),
            Register16::DE => self.registers.get_de(),
            Register16::HL => self.registers.get_hl(),
            Register16::SP => self.registers.get_sp(),
        }
    }

    fn set_register16(&mut self, register: &Register16, value: u16) {
        match register {
            Register16::BC => self.registers.set_bc(value),
            Register16::DE => self.registers.set_de(value),
            Register16::HL => self.registers.set_hl(value),
            Register16::SP => self.registers.set_sp(value),
        }
    }

    /// Sets the flags affected by an SP plus immediate offset operation.
    fn set_flags_sp_plus_immediate(&mut self, sp: u16, offset: i8) {
        let offset = offset as u8;

        self.registers.set_zero(false);
        self.registers.set_subtract(false);
        self.registers
            .set_half_carry((sp & 0x000F) + (offset as u16 & 0x000F) > 0x000F);

        self.registers
            .set_carry((sp & 0x00FF) + offset as u16 > 0x00FF);
    }

    /// Enables IME if the previous instruction scheduled it.
    fn check_ime(&mut self, pending_ime: bool) {
        if pending_ime {
            self.ime = true;
            self.ime_schedule = false;
        }
    }

    /// Executes the instruction and returns its T-Cycles.
    fn execute(&mut self, instruction: Instruction) -> u8 {
        self.check_ime(self.ime_schedule);

        match instruction {
            Instruction::Nop => 4,

            Instruction::Load(instruction) => self.execute_load(instruction),

            Instruction::Arithmetic(instruction) => self.execute_arithmetic(instruction),

            Instruction::Stack(instruction) => self.execute_stack(instruction),

            Instruction::Rotation(instruction) => self.execute_rotation(instruction),

            Instruction::Control(instruction) => self.execute_control(instruction),
            Instruction::Cb => {
                let opcode = self.fetch();
                let instruction = decode_cb(opcode);

                self.execute_cb(instruction)
            }
        }
    }

    pub fn take_serial_output(&mut self) -> Option<u8> {
        self.bus.take_serial_output()
    }

    /// Executes one CPU step and returns the number of T-Cycles consumed.
    pub fn step(&mut self) -> u8 {
        let pc = self.registers.get_pc();
        info!(
            "T-Cycles: {} | Executing: {:#04x}",
            self.total_t_cycles,
            self.bus.read(pc)
        );

        let prev_ticks = self.get_total_ticks();

        if self.stopped {
            return NON_CYCLES;
        }
        if self.halted {
            return self.step_halted();
        }

        if self.ime
            && let Some(interruption) = self.check_interruption()
        {
            let t_cycles_interrupt = self.handle_interruption(interruption);

            let current_ticks = self.get_total_ticks();
            assert_eq!(current_ticks - prev_ticks, t_cycles_interrupt as u64);

            return t_cycles_interrupt;
        }
        let opcode = self.fetch();
        let instruction = decode(opcode);

        let real_t = self.execute(instruction);

        let current_ticks = self.get_total_ticks();

        assert_eq!(current_ticks - prev_ticks, real_t as u64);

        real_t
    }

    fn step_halted(&mut self) -> u8 {
        let prev_ticks = self.get_total_ticks();

        let Some(interruption) = self.check_interruption() else {
            // HALT without interruptions consumes 4 T-Cycles.
            self.tick_internal(1);
            let current_ticks = self.get_total_ticks();
            assert_eq!(current_ticks - prev_ticks, HALT_CYCLES as u64);

            return HALT_CYCLES;
        };

        self.halted = false;

        if self.ime {
            let t_cycles_interrupt = self.handle_interruption(interruption);
            let current_ticks = self.get_total_ticks();

            assert_eq!(current_ticks - prev_ticks, t_cycles_interrupt as u64);

            t_cycles_interrupt
        } else {
            // Wake from HALT without servicing the interrupt.
            NON_CYCLES
        }
    }
}
