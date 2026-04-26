#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiRequest {
    None,
}

pub fn update() -> UiRequest {
    UiRequest::None
}
