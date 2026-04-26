use crate::engine::TableBridgeState;
use crate::engine::math::Vec2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TableMessage {
    LeftFlipperPressed,
    LeftFlipperReleased,
    RightFlipperPressed,
    RightFlipperReleased,
    PlungerPressed,
    PlungerReleased,
    StartGame,
    Nudge(Vec2),
    Pause,
    Resume,
}

impl TableMessage {
    pub fn from_bridge_state(current: &TableBridgeState, previous: &TableBridgeState) -> Vec<Self> {
        let mut messages = Vec::new();

        if current.left_flipper != previous.left_flipper {
            messages.push(if current.left_flipper {
                Self::LeftFlipperPressed
            } else {
                Self::LeftFlipperReleased
            });
        }

        if current.right_flipper != previous.right_flipper {
            messages.push(if current.right_flipper {
                Self::RightFlipperPressed
            } else {
                Self::RightFlipperReleased
            });
        }

        if current.plunger_pulling != previous.plunger_pulling {
            messages.push(if current.plunger_pulling {
                Self::PlungerPressed
            } else {
                Self::PlungerReleased
            });
        }

        if current.pending_start {
            messages.push(Self::StartGame);
        }

        if let Some((x, y)) = current.pending_nudge {
            messages.push(Self::Nudge(Vec2::new(x, y)));
        }

        messages
    }
}
