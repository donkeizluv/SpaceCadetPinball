use crate::engine::TableBridgeState;
use crate::engine::math::Vec2;

#[derive(Debug, Clone, Copy, Default)]
pub struct TableInputState {
    pub left_flipper: bool,
    pub right_flipper: bool,
    pub plunger_pulling: bool,
    pub pending_start: bool,
    pub pending_nudge: Option<Vec2>,
    pub ticks: u64,
}

impl From<&TableBridgeState> for TableInputState {
    fn from(value: &TableBridgeState) -> Self {
        Self {
            left_flipper: value.left_flipper,
            right_flipper: value.right_flipper,
            plunger_pulling: value.plunger_pulling,
            pending_start: value.pending_start,
            pending_nudge: value.pending_nudge.map(|(x, y)| Vec2::new(x, y)),
            ticks: value.input_ticks,
        }
    }
}
