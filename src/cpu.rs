#![allow(dead_code)]

pub struct Cpu {
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

impl Cpu {
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
}

#[cfg(test)]
mod tests {
    use rand::RngExt;

    use super::*;

    #[test]
    fn set_and_get_af() {
        let mut cpu = Cpu::new();

        let mut rng = rand::rng();
        let value: u16 = rng.random();

        cpu.set_af(value);

        assert_eq!(cpu.get_af(), value);
    }

    #[test]
    fn set_and_get_bc() {
        let mut cpu = Cpu::new();

        let mut rng = rand::rng();
        let value: u16 = rng.random();

        cpu.set_bc(value);

        assert_eq!(cpu.get_bc(), value);
    }

    #[test]
    fn set_and_get_de() {
        let mut cpu = Cpu::new();

        let mut rng = rand::rng();
        let value: u16 = rng.random();

        cpu.set_de(value);

        assert_eq!(cpu.get_de(), value);
    }

    #[test]
    fn set_and_get_hl() {
        let mut cpu = Cpu::new();

        let mut rng = rand::rng();
        let value: u16 = rng.random();

        cpu.set_hl(value);

        assert_eq!(cpu.get_hl(), value);
    }
}
