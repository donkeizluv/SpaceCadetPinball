use sdl2::keyboard::Keycode;

use super::input;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyBinding {
    pub keycode: Keycode,
    pub action: &'static str,
}

#[derive(Debug, Clone)]
pub struct InputBindings {
    keyboard: Vec<KeyBinding>,
}

impl InputBindings {
    pub fn action_for_key(&self, keycode: Keycode) -> Option<&'static str> {
        self.keyboard
            .iter()
            .find(|binding| binding.keycode == keycode)
            .map(|binding| binding.action)
    }

    pub fn keyboard_bindings(&self) -> &[KeyBinding] {
        &self.keyboard
    }
}

impl Default for InputBindings {
    fn default() -> Self {
        Self {
            keyboard: vec![
                KeyBinding {
                    keycode: Keycode::LShift,
                    action: input::ACTION_LEFT_FLIPPER,
                },
                KeyBinding {
                    keycode: Keycode::RShift,
                    action: input::ACTION_RIGHT_FLIPPER,
                },
                KeyBinding {
                    keycode: Keycode::Space,
                    action: input::ACTION_PLUNGER_PULL,
                },
                KeyBinding {
                    keycode: Keycode::Return,
                    action: input::ACTION_START,
                },
                KeyBinding {
                    keycode: Keycode::Backspace,
                    action: input::ACTION_BACK,
                },
                KeyBinding {
                    keycode: Keycode::Left,
                    action: input::ACTION_NUDGE_LEFT,
                },
                KeyBinding {
                    keycode: Keycode::Right,
                    action: input::ACTION_NUDGE_RIGHT,
                },
                KeyBinding {
                    keycode: Keycode::Up,
                    action: input::ACTION_NUDGE_UP,
                },
                KeyBinding {
                    keycode: Keycode::Down,
                    action: input::ACTION_NUDGE_DOWN,
                },
            ],
        }
    }
}
