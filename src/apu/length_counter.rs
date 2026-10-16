pub(super) const LENGTH_COUNTER_MAX: u16 = 64;

pub(super) struct LengthCounter {
    max_length: u16,
    counter: u16,
}

impl LengthCounter {
    pub(super) fn new(max_length: u16) -> Self {
        Self {
            max_length,
            counter: max_length,
        }
    }

    pub(super) fn load(&mut self, value: u16) {
        self.counter = self.max_length - value;
    }

    pub(super) fn reload(&mut self) {
        if self.is_zero() {
            self.counter = self.max_length;
        }
    }

    pub(super) fn tick(&mut self) -> bool {
        if self.counter == 0 {
            return true;
        }

        self.counter -= 1;
        self.counter == 0
    }

    pub(super) fn is_zero(&self) -> bool {
        self.counter == 0
    }

    #[cfg(test)]
    pub(super) fn get_counter(&self) -> u16 {
        self.counter
    }
}

impl Default for LengthCounter {
    fn default() -> Self {
        Self::new(LENGTH_COUNTER_MAX)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MAX_LENGTH: u16 = 64;

    #[test]
    fn counter_starts_at_max_length() {
        let counter = LengthCounter::new(MAX_LENGTH);

        assert_eq!(counter.get_counter(), MAX_LENGTH);
        assert!(!counter.is_zero());
    }

    #[test]
    fn load_sets_counter_from_length_value() {
        let mut counter = LengthCounter::new(MAX_LENGTH);
        let length = 20;

        counter.load(length);

        assert_eq!(counter.get_counter(), MAX_LENGTH - length);
    }

    #[test]
    fn tick_decrements_counter() {
        let mut counter = LengthCounter::new(MAX_LENGTH);

        counter.tick();

        assert_eq!(counter.get_counter(), MAX_LENGTH - 1);
    }

    #[test]
    fn tick_returns_true_when_counter_reaches_zero() {
        let mut counter = LengthCounter::new(MAX_LENGTH);
        counter.load(MAX_LENGTH - 1);

        assert!(counter.tick());
        assert!(counter.is_zero());
    }

    #[test]
    fn tick_keeps_zero_counter_at_zero() {
        let mut counter = LengthCounter::new(MAX_LENGTH);
        counter.load(MAX_LENGTH);

        assert!(counter.tick());
        assert_eq!(counter.get_counter(), 0);
    }
}
