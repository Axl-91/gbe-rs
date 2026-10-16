const DAC_MASK: u8 = 0xF8;
const DIRECTION_MASK: u8 = 0b0000_1000;
const PERIOD_MASK: u8 = 0b0000_0111;
const INITIAL_VOLUME_SHIFT: u32 = 4;
const MAX_VOLUME: u8 = 0x0F;
const ZERO_PERIOD: u8 = 8;

pub(super) fn is_dac_enabled(register: u8) -> bool {
    register & DAC_MASK != 0
}

pub(super) fn get_period(register: u8) -> u8 {
    let period = register & PERIOD_MASK;

    if period == 0 { ZERO_PERIOD } else { period }
}

pub(super) fn trigger(register: u8, volume: &mut u8, timer: &mut u8) {
    *volume = register >> INITIAL_VOLUME_SHIFT;
    *timer = get_period(register);
}

pub(super) fn clock(register: u8, active: bool, volume: &mut u8, timer: &mut u8) {
    if !active || register & PERIOD_MASK == 0 {
        return;
    }

    *timer -= 1;

    if *timer > 0 {
        return;
    }

    *timer = get_period(register);

    let increasing = register & DIRECTION_MASK != 0;

    if increasing && *volume < MAX_VOLUME {
        *volume += 1;
    } else if !increasing && *volume > 0 {
        *volume -= 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_period_does_not_change_volume() {
        let register = 0x08;
        let mut volume = 0;
        let mut timer = get_period(register);

        for _ in 0..16 {
            clock(register, true, &mut volume, &mut timer);
        }

        assert_eq!(volume, 0);
    }
}
