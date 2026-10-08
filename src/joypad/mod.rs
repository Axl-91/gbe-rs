//! Game Boy joypad (P1/JOYP) register.
//!
//! The joypad exposes two groups of four buttons (the D-pad and the
//! action/menu buttons) through a single register. A game selects which
//! group it wants to read via bits 4-5, and reads the state of that
//! group's four buttons back via bits 0-3. Both groups use active-low
//! logic: a cleared bit means the corresponding button is pressed.

#[cfg(test)]
mod tests;

const SELECT_MASK: u8 = 0x30;
const INACTIVE: u8 = 0x0F;

const BOTH_SELECTED: u8 = 0x00;
const BUTTONS_SELECTED: u8 = 0x10;
const DPAD_SELECTED: u8 = 0x20;

/// A single Game Boy button.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Button {
    Right,
    Left,
    Up,
    Down,
    A,
    B,
    Select,
    Start,
}

impl Button {
    /// Returns the bit position of this button within its group's nibble.
    fn bit(&self) -> u8 {
        match self {
            Button::Right | Button::A => 0,
            Button::Left | Button::B => 1,
            Button::Up | Button::Select => 2,
            Button::Down | Button::Start => 3,
        }
    }

    /// Returns whether this button belongs to the D-pad group.
    fn is_dpad(&self) -> bool {
        matches!(
            self,
            Button::Right | Button::Left | Button::Up | Button::Down
        )
    }
}

/// Represents the Game Boy joypad register and button state.
pub struct Joypad {
    select: u8,
    dpad: u8,
    buttons: u8,
    state: u8,
}

impl Joypad {
    /// Creates a new joypad with no group selected and no buttons pressed.
    pub fn new() -> Self {
        Self {
            select: BOTH_SELECTED,
            dpad: INACTIVE,
            buttons: INACTIVE,
            state: INACTIVE,
        }
    }

    /// Reads the current value of the `P1`/`JOYP` register.
    pub fn read(&self) -> u8 {
        0xC0 | (self.select & SELECT_MASK) | self.state
    }

    /// Calculates the current state of P10-P13 from the selected buttons.
    fn calculate_state(&self) -> u8 {
        match self.select & SELECT_MASK {
            BOTH_SELECTED => self.dpad & self.buttons,
            BUTTONS_SELECTED => self.buttons,
            DPAD_SELECTED => self.dpad,
            _ => INACTIVE,
        }
    }

    /// Updates the stored P10-P13 state and returns whether a
    /// 1 -> 0 transition occurred.
    fn update_state(&mut self) -> bool {
        let previous_state = self.state;
        let current_state = self.calculate_state();

        self.state = current_state;

        (previous_state & !current_state) != 0
    }

    /// Updates which button group(s) are selected.
    pub fn write(&mut self, value: u8) -> bool {
        self.select = value & SELECT_MASK;
        self.update_state()
    }

    /// Marks a button as pressed.
    pub fn press(&mut self, button: Button) -> bool {
        let mask = !(1 << button.bit());

        if button.is_dpad() {
            self.dpad &= mask;
        } else {
            self.buttons &= mask;
        }

        self.update_state()
    }

    /// Marks a button as released.
    pub fn release(&mut self, button: Button) {
        let mask = 1 << button.bit();

        if button.is_dpad() {
            self.dpad |= mask;
        } else {
            self.buttons |= mask;
        }
        self.update_state();
    }

    /// Returns whether any of the currently selected button lines
    /// (P10-P13) reads as pressed.
    pub fn is_joypad_active(&self) -> bool {
        self.state != INACTIVE
    }
}

impl Default for Joypad {
    fn default() -> Self {
        Self::new()
    }
}
