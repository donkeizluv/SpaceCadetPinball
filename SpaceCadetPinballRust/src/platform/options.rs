use std::env;
use std::path::PathBuf;

use super::input_bindings::InputBindings;

#[derive(Debug, Clone)]
pub struct Options {
    pub start_fullscreen: bool,
    pub show_fps: bool,
    pub input_bindings: InputBindings,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            start_fullscreen: false,
            show_fps: false,
            input_bindings: InputBindings::default(),
        }
    }
}

pub fn default_options_path() -> PathBuf {
    env::current_dir()
        .unwrap_or_default()
        .join("pinball-options.json")
}
