pub struct Registers {
    a: u8,
    f: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    sp: u16,
    pc: u16,
}

const SHIFT_ZERO: u8 = 7;
const SHIFT_SUB: u8 = 6;
const SHIFT_HCARRY: u8 = 5;
const SHIFT_CARRY: u8 = 4;

impl Registers {
    pub fn new() -> Self {
        Self {
            a: 0,
            f: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            sp: 0,
            pc: 0,
        }
    }

    pub fn get_af(&self) -> u16 {
        ((self.a as u16) << 8) | self.f as u16
    }

    pub fn set_af(&mut self, value: u16) {
        self.a = (value >> 8) as u8;
        self.f = (value & 0x00FF) as u8;
    }

    pub fn get_bc(&self) -> u16 {
        ((self.b as u16) << 8) | self.c as u16
    }

    pub fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = (value & 0x00FF) as u8;
    }

    pub fn get_de(&self) -> u16 {
        ((self.d as u16) << 8) | self.e as u16
    }

    pub fn set_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = (value & 0x00FF) as u8;
    }

    pub fn get_hl(&self) -> u16 {
        ((self.h as u16) << 8) | self.l as u16
    }

    pub fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = (value & 0x00FF) as u8;
    }

    pub fn get_zero(&self) -> bool {
        ((self.f >> SHIFT_ZERO) & 0b1) != 0
    }

    pub fn get_subtract(&self) -> bool {
        ((self.f >> SHIFT_SUB) & 0b1) != 0
    }

    pub fn get_half_carry(&self) -> bool {
        ((self.f >> SHIFT_HCARRY) & 0b1) != 0
    }

    pub fn get_carry(&self) -> bool {
        ((self.f >> SHIFT_CARRY) & 0b1) != 0
    }

    pub fn set_zero(&mut self, is_enable: bool) {
        let mask: u8 = 1 << SHIFT_ZERO;
        if is_enable {
            self.f |= mask
        } else {
            self.f &= !mask
        }
    }

    pub fn set_subtract(&mut self, is_enable: bool) {
        let mask: u8 = 1 << SHIFT_SUB;
        if is_enable {
            self.f |= mask
        } else {
            self.f &= !mask
        }
    }

    pub fn set_half_carry(&mut self, is_enable: bool) {
        let mask: u8 = 1 << SHIFT_HCARRY;
        if is_enable {
            self.f |= mask
        } else {
            self.f &= !mask
        }
    }

    pub fn set_carry(&mut self, is_enable: bool) {
        let mask: u8 = 1 << SHIFT_CARRY;
        if is_enable {
            self.f |= mask
        } else {
            self.f &= !mask
        }
    }
    pub fn get_pc(&self) -> u16 {
        self.pc
    }

    pub fn set_pc(&mut self, value: u16) {
        self.pc = value;
    }

    pub fn get_sp(&self) -> u16 {
        self.sp
    }

    pub fn set_sp(&mut self, value: u16) {
        self.sp = value;
    }
}

#[cfg(test)]
mod tests {
    mod registers {
        use super::super::*;
        use rand::RngExt;

        #[test]
        fn set_and_get_af() {
            let mut cpu = Registers::new();

            let mut rng = rand::rng();
            let value: u16 = rng.random();

            cpu.set_af(value);

            assert_eq!(cpu.get_af(), value);
        }

        #[test]
        fn set_and_get_bc() {
            let mut cpu = Registers::new();

            let mut rng = rand::rng();
            let value: u16 = rng.random();

            cpu.set_bc(value);

            assert_eq!(cpu.get_bc(), value);
        }

        #[test]
        fn set_and_get_de() {
            let mut cpu = Registers::new();

            let mut rng = rand::rng();

            let value: u16 = rng.random();

            cpu.set_de(value);

            assert_eq!(cpu.get_de(), value);
        }

        #[test]
        fn set_and_get_hl() {
            let mut cpu = Registers::new();

            let mut rng = rand::rng();
            let value: u16 = rng.random();

            cpu.set_hl(value);

            assert_eq!(cpu.get_hl(), value);
        }

        #[test]
        fn set_and_get_pc() {
            let mut rng = rand::rng();

            let value: u16 = rng.random();

            let mut cpu = Registers::new();
            cpu.set_pc(value);

            assert_eq!(cpu.get_pc(), value);
        }

        #[test]
        fn set_and_get_sp() {
            let mut rng = rand::rng();

            let value: u16 = rng.random();

            let mut cpu = Registers::new();
            cpu.set_sp(value);

            assert_eq!(cpu.get_sp(), value);
        }
    }
    mod flags {
        use super::super::*;

        #[test]
        fn set_zero() {
            let mut cpu = Registers::new();

            cpu.set_zero(true);
            assert!(cpu.get_zero());

            cpu.set_zero(false);
            assert!(!cpu.get_zero());
        }

        #[test]
        fn set_subtract() {
            let mut cpu = Registers::new();

            cpu.set_subtract(true);
            assert!(cpu.get_subtract());

            cpu.set_subtract(false);
            assert!(!cpu.get_subtract());
        }

        #[test]
        fn set_half_carry() {
            let mut cpu = Registers::new();

            cpu.set_half_carry(true);
            assert!(cpu.get_half_carry());

            cpu.set_half_carry(false);
            assert!(!cpu.get_half_carry());
        }

        #[test]
        fn set_carry() {
            let mut cpu = Registers::new();

            cpu.set_carry(true);
            assert!(cpu.get_carry());

            cpu.set_carry(false);
            assert!(!cpu.get_carry());
        }

        #[test]
        fn setting_flags_does_not_modify_other_flags() {
            let mut cpu = Registers::new();

            cpu.set_zero(true);
            assert!(cpu.get_zero());
            assert!(!cpu.get_subtract());

            assert!(!cpu.get_half_carry());
            assert!(!cpu.get_carry());

            cpu.set_subtract(true);
            assert!(cpu.get_zero());
            assert!(cpu.get_subtract());
            assert!(!cpu.get_half_carry());
            assert!(!cpu.get_carry());

            cpu.set_half_carry(true);
            assert!(cpu.get_zero());
            assert!(cpu.get_subtract());
            assert!(cpu.get_half_carry());
            assert!(!cpu.get_carry());

            cpu.set_carry(true);
            assert!(cpu.get_zero());
            assert!(cpu.get_subtract());
            assert!(cpu.get_half_carry());
            assert!(cpu.get_carry());
        }

        #[test]
        fn clearing_flags_does_not_modify_other_flags() {
            let mut cpu = Registers::new();

            cpu.set_zero(true);
            cpu.set_subtract(true);
            cpu.set_half_carry(true);
            cpu.set_carry(true);

            cpu.set_subtract(false);

            assert!(cpu.get_zero());
            assert!(!cpu.get_subtract());
            assert!(cpu.get_half_carry());
            assert!(cpu.get_carry());
        }
    }
}
