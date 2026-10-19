const FREQUENCY_TIMER_MULTIPLIER: u16 = 4;

pub(super) const FREQUENCY_MAX: u16 = 0x07FF;
pub(super) const DUTY_STEPS: u8 = 8;

const FREQUENCY_HIGH_MASK: u8 = 0b0000_0111;

const DUTY_MASK: u8 = 0b1100_0000;
const DUTY_SHIFT: u32 = 6;

const DUTY_PATTERNS: [u8; 4] = [0b1000_0000, 0b1000_0001, 0b1110_0001, 0b0111_1110];

pub(super) fn get_frequency(low: u8, high: u8) -> u16 {
    let low = u16::from(low);
    let high = u16::from(high & FREQUENCY_HIGH_MASK);

    (high << u8::BITS) | low
}

pub(super) fn get_frequency_period(frequency: u16) -> u16 {
    (FREQUENCY_MAX + 1 - frequency) * FREQUENCY_TIMER_MULTIPLIER
}

pub(super) fn get_duty_output(duty_register: u8, duty_position: u8) -> bool {
    let duty = usize::from((duty_register & DUTY_MASK) >> DUTY_SHIFT);
    let pattern = DUTY_PATTERNS[duty];
    let bit = (pattern >> duty_position) & 1;

    bit != 0
}

pub(super) fn tick(
    active: bool,
    frequency_timer: &mut u16,
    frequency_period: u16,
    duty_position: &mut u8,
    initial_output_suppressed: &mut bool,
) {
    if !active {
        return;
    }

    *frequency_timer -= 1;

    if *frequency_timer == 0 {
        *frequency_timer = frequency_period;
        *duty_position = (*duty_position + 1) % DUTY_STEPS;
        *initial_output_suppressed = false;
    }
}
