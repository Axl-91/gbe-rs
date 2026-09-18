use super::*;
use rand::RngExt;

mod select {
    use super::*;

    #[test]
    fn no_group_selected_initially() {
        let joypad = Joypad::new();
        assert_eq!(joypad.read() & 0x0F, 0x0F);
    }

    #[test]
    fn write_only_uses_bits_4_and_5() {
        let mut rng = rand::rng();
        let garbage: u8 = rng.random();

        let mut joypad = Joypad::new();
        joypad.write(garbage);

        let expected = garbage & 0x30;
        assert_eq!(joypad.select, expected);
    }

    #[test]
    fn unused_bits_always_read_as_set() {
        let mut rng = rand::rng();
        let value: u8 = rng.random();

        let mut joypad = Joypad::new();
        joypad.write(value);

        assert_eq!(joypad.read() & 0xC0, 0xC0);
    }
}

mod dpad {
    use super::*;

    #[test]
    fn no_buttons_pressed_initially() {
        let mut joypad = Joypad::new();
        joypad.write(0x20);

        assert_eq!(joypad.read() & 0x0F, 0x0F);
    }

    #[test]
    fn press_clears_the_corresponding_bit() {
        let mut joypad = Joypad::new();
        joypad.write(0x20);

        joypad.press(Button::Up);

        assert_eq!(joypad.read() & 0x0F, 0b1011);
    }

    #[test]
    fn release_sets_the_corresponding_bit() {
        let mut joypad = Joypad::new();
        joypad.write(0x20);

        joypad.press(Button::Up);
        joypad.release(Button::Up);

        assert_eq!(joypad.read() & 0x0F, 0x0F);
    }

    #[test]
    fn pressing_dpad_button_does_not_affect_buttons_group() {
        let mut joypad = Joypad::new();

        joypad.press(Button::Down);

        joypad.write(0x10);
        assert_eq!(joypad.read() & 0x0F, 0x0F);
    }
}

mod buttons {
    use super::*;

    #[test]
    fn no_buttons_pressed_initially() {
        let mut joypad = Joypad::new();
        joypad.write(0x10);

        assert_eq!(joypad.read() & 0x0F, 0x0F);
    }

    #[test]
    fn press_clears_the_corresponding_bit() {
        let mut joypad = Joypad::new();
        joypad.write(0x10);

        joypad.press(Button::A);

        assert_eq!(joypad.read() & 0x0F, 0b1110);
    }

    #[test]
    fn release_sets_the_corresponding_bit() {
        let mut joypad = Joypad::new();
        joypad.write(0x10);

        joypad.press(Button::A);
        joypad.release(Button::A);

        assert_eq!(joypad.read() & 0x0F, 0x0F);
    }

    #[test]
    fn pressing_button_does_not_affect_dpad_group() {
        let mut joypad = Joypad::new();

        joypad.press(Button::Start);

        joypad.write(0x20); // selecciona solo d-pad
        assert_eq!(joypad.read() & 0x0F, 0x0F);
    }
}

mod both_groups_selected {
    use super::*;

    #[test]
    fn reads_as_not_pressed_when_nothing_is_pressed() {
        let mut joypad = Joypad::new();
        joypad.write(0x00);

        assert_eq!(joypad.read() & 0x0F, 0x0F);
    }

    #[test]
    fn reflects_a_press_in_either_group() {
        let mut joypad = Joypad::new();

        joypad.press(Button::A);
        joypad.write(0x00);

        assert_eq!(joypad.read() & 0x0F, 0b1110);
    }

    #[test]
    fn combines_presses_from_both_groups() {
        let mut joypad = Joypad::new();

        joypad.press(Button::A);
        joypad.press(Button::Left);
        joypad.write(0x00);

        assert_eq!(joypad.read() & 0x0F, 0b1100);
    }

    #[test]
    fn same_bit_position_in_both_groups_requires_both_pressed() {
        let mut joypad = Joypad::new();
        joypad.write(0x00);

        joypad.press(Button::Right);
        assert_eq!(joypad.read() & 0x01, 0x00);

        joypad.press(Button::A);
        assert_eq!(joypad.read() & 0x01, 0x00);
    }
}

mod is_joypad_active {
    use super::*;

    #[test]
    fn false_when_nothing_pressed() {
        let joypad = Joypad::new();

        assert!(!joypad.is_joypad_active());
    }

    #[test]
    fn false_when_pressed_button_group_is_not_selected() {
        let mut joypad = Joypad::new();
        joypad.write(0x20); // selecciona d-pad

        joypad.press(Button::A);

        assert!(!joypad.is_joypad_active());
    }

    #[test]
    fn true_when_pressed_button_group_is_selected() {
        let mut joypad = Joypad::new();
        joypad.write(0x10);

        joypad.press(Button::B);

        assert!(joypad.is_joypad_active());
    }

    #[test]
    fn true_when_no_group_explicitly_selected_but_both_read() {
        let mut joypad = Joypad::new();
        joypad.write(0x00);

        joypad.press(Button::Down);

        assert!(joypad.is_joypad_active());
    }
}
