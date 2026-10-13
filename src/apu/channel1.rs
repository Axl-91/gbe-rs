const NR10_ADDRESS: u16 = 0xFF10;
const NR11_ADDRESS: u16 = 0xFF11;
const NR12_ADDRESS: u16 = 0xFF12;
const NR13_ADDRESS: u16 = 0xFF13;
const NR14_ADDRESS: u16 = 0xFF14;

const NR10_READ_MASK: u8 = 0x80;
const NR11_READ_MASK: u8 = 0x3F;
const NR13_READ_VALUE: u8 = 0xFF;
const NR14_READ_MASK: u8 = 0xBF;

const NR14_TRIGGER_MASK: u8 = 0x80;
const NR12_DAC_MASK: u8 = 0xF8;

pub(super) struct Channel1 {
    nr10: u8,
    nr11: u8,
    nr12: u8,
    nr13: u8,
    nr14: u8,
    active: bool,
}

impl Channel1 {
    pub fn new() -> Self {
        Self {
            nr10: 0,
            nr11: 0,
            nr12: 0,
            nr13: 0,
            nr14: 0,
            active: false,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            NR10_ADDRESS => self.nr10 | NR10_READ_MASK,
            NR11_ADDRESS => self.nr11 | NR11_READ_MASK,
            NR12_ADDRESS => self.nr12,
            NR13_ADDRESS => NR13_READ_VALUE,
            NR14_ADDRESS => self.nr14 | NR14_READ_MASK,
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            NR10_ADDRESS => self.nr10 = value,
            NR11_ADDRESS => self.nr11 = value,
            NR12_ADDRESS => {
                self.nr12 = value;

                if !self.dac_enabled() {
                    self.active = false;
                }
            }
            NR13_ADDRESS => self.nr13 = value,
            NR14_ADDRESS => {
                self.nr14 = value & !NR14_TRIGGER_MASK;

                if value & NR14_TRIGGER_MASK != 0 {
                    self.trigger();
                }
            }
            _ => {}
        }
    }

    fn dac_enabled(&self) -> bool {
        self.nr12 & NR12_DAC_MASK != 0
    }

    fn trigger(&mut self) {
        self.active = self.dac_enabled();
    }

    pub fn is_active(&self) -> bool {
        self.active
    }
}

impl Default for Channel1 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_return_expected_values_after_writes() {
        let mut channel = Channel1::new();

        let nr10_value = 0b0101_0011;
        let nr11_value = 0b1000_1010;
        let nr12_value = 0b1100_0101;
        let nr13_value = 0b0110_1010;
        let nr14_value = 0b0100_0011;

        channel.write(NR10_ADDRESS, nr10_value);
        channel.write(NR11_ADDRESS, nr11_value);
        channel.write(NR12_ADDRESS, nr12_value);
        channel.write(NR13_ADDRESS, nr13_value);
        channel.write(NR14_ADDRESS, nr14_value);

        assert_eq!(channel.read(NR10_ADDRESS), nr10_value | NR10_READ_MASK);
        assert_eq!(channel.read(NR11_ADDRESS), nr11_value | NR11_READ_MASK);
        assert_eq!(channel.read(NR12_ADDRESS), nr12_value);
        assert_eq!(channel.read(NR13_ADDRESS), NR13_READ_VALUE);
        assert_eq!(channel.read(NR14_ADDRESS), nr14_value | NR14_READ_MASK);
    }

    #[test]
    fn channel_starts_inactive() {
        let channel = Channel1::new();

        assert!(!channel.is_active());
    }

    #[test]
    fn trigger_activates_channel_when_dac_is_enabled() {
        let mut channel = Channel1::new();

        channel.write(NR12_ADDRESS, NR12_DAC_MASK);
        channel.write(NR14_ADDRESS, NR14_TRIGGER_MASK);

        assert!(channel.is_active());
    }

    #[test]
    fn trigger_does_not_activate_channel_when_dac_is_disabled() {
        let mut channel = Channel1::new();

        channel.write(NR12_ADDRESS, 0);
        channel.write(NR14_ADDRESS, NR14_TRIGGER_MASK);

        assert!(!channel.is_active());
    }

    #[test]
    fn disabling_dac_deactivates_channel() {
        let mut channel = Channel1::new();

        channel.write(NR12_ADDRESS, NR12_DAC_MASK);
        channel.write(NR14_ADDRESS, NR14_TRIGGER_MASK);
        channel.write(NR12_ADDRESS, 0);

        assert!(!channel.is_active());
    }
}
