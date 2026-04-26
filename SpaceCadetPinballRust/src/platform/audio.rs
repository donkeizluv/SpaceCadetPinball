#[derive(Debug, Clone, Copy, Default)]
pub struct AudioState {
    pub enabled: bool,
}

pub fn initialize() -> AudioState {
    AudioState { enabled: false }
}
