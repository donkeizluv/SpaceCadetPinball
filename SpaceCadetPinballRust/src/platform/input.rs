use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;

use super::input_bindings::InputBindings;

pub const ACTION_LEFT_FLIPPER: &str = "left_flipper";
pub const ACTION_RIGHT_FLIPPER: &str = "right_flipper";
pub const ACTION_PLUNGER_PULL: &str = "plunger_pull";
pub const ACTION_MOUSE_LEFT: &str = "mouse_left";
pub const ACTION_START: &str = "start";
pub const ACTION_BACK: &str = "back";
pub const ACTION_NUDGE_LEFT: &str = "nudge_left";
pub const ACTION_NUDGE_RIGHT: &str = "nudge_right";
pub const ACTION_NUDGE_UP: &str = "nudge_up";
pub const ACTION_NUDGE_DOWN: &str = "nudge_down";
pub const ACTION_NUDGE: &str = "nudge";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlatformEvent {
    ExitRequested,
    ToggleFullscreen,
    ActionDown(&'static str),
    ActionUp(&'static str),
    Nudge { dx: f32, dy: f32 },
}

pub fn is_impulse_action(action: &'static str) -> bool {
    matches!(
        action,
        ACTION_START
            | ACTION_BACK
            | ACTION_NUDGE_LEFT
            | ACTION_NUDGE_RIGHT
            | ACTION_NUDGE_UP
            | ACTION_NUDGE_DOWN
            | ACTION_NUDGE
    )
}

pub fn translate_event(event: &Event, bindings: &InputBindings) -> Option<PlatformEvent> {
    match event {
        Event::Quit { .. } => Some(PlatformEvent::ExitRequested),
        Event::KeyDown {
            keycode: Some(Keycode::Escape),
            repeat: false,
            ..
        } => Some(PlatformEvent::ExitRequested),
        Event::KeyDown {
            keycode: Some(Keycode::F11),
            repeat: false,
            ..
        } => Some(PlatformEvent::ToggleFullscreen),
        Event::KeyDown {
            keycode: Some(keycode),
            repeat: false,
            ..
        } => translate_key_down(*keycode, bindings),
        Event::KeyUp {
            keycode: Some(keycode),
            repeat: false,
            ..
        } => bindings
            .action_for_key(*keycode)
            .map(PlatformEvent::ActionUp),
        Event::MouseButtonDown {
            mouse_btn: MouseButton::Left,
            ..
        } => Some(PlatformEvent::ActionDown(ACTION_MOUSE_LEFT)),
        Event::MouseButtonUp {
            mouse_btn: MouseButton::Left,
            ..
        } => Some(PlatformEvent::ActionUp(ACTION_MOUSE_LEFT)),
        _ => None,
    }
}

fn translate_key_down(keycode: Keycode, bindings: &InputBindings) -> Option<PlatformEvent> {
    let action = bindings.action_for_key(keycode)?;
    if let Some((dx, dy)) = nudge_vector_for_action(action) {
        Some(PlatformEvent::Nudge { dx, dy })
    } else {
        Some(PlatformEvent::ActionDown(action))
    }
}

fn nudge_vector_for_action(action: &'static str) -> Option<(f32, f32)> {
    match action {
        ACTION_NUDGE_LEFT => Some((-0.02, 0.0)),
        ACTION_NUDGE_RIGHT => Some((0.02, 0.0)),
        ACTION_NUDGE_UP => Some((0.0, 0.02)),
        ACTION_NUDGE_DOWN => Some((0.0, -0.02)),
        _ => None,
    }
}
