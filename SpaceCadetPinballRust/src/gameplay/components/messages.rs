use crate::engine::TableBridgeState;
use crate::engine::math::Vec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MessageCode {
    TFlipperNull,
    TFlipperExtend,
    TFlipperRetract,
    TLightTurnOff,
    TLightTurnOn,
    TLightGetLightOnFlag,
    TLightGetFlasherOnFlag,
    TLightFlasherStart,
    TLightApplyMultDelay,
    TLightApplyDelay,
    TLightFlasherStartTimed,
    TLightTurnOffTimed,
    TLightTurnOnTimed,
    TLightSetOnStateBmpIndex,
    TLightIncOnStateBmpIndex,
    TLightDecOnStateBmpIndex,
    TLightResetTimed,
    TLightFlasherStartTimedThenStayOn,
    TLightFlasherStartTimedThenStayOff,
    TLightToggleValue,
    TLightResetAndToggleValue,
    TLightResetAndTurnOn,
    TLightResetAndTurnOff,
    TLightToggle,
    TLightResetAndToggle,
    TLightSetMessageField,
    TLightFtTmpOverrideOn,
    TLightFtTmpOverrideOff,
    TLightFtResetOverride,
    TLightGroupStepBackward,
    TLightGroupStepForward,
    TLightGroupAnimationBackward,
    TLightGroupAnimationForward,
    TLightGroupLightShowAnimation,
    TLightGroupGameOverAnimation,
    TLightGroupRandomAnimationSaturation,
    TLightGroupRandomAnimationDesaturation,
    TLightGroupOffsetAnimationForward,
    TLightGroupOffsetAnimationBackward,
    TLightGroupReset,
    TLightGroupTurnOnAtIndex,
    TLightGroupTurnOffAtIndex,
    TLightGroupGetOnCount,
    TLightGroupGetLightCount,
    TLightGroupGetMessage2,
    TLightGroupGetAnimationFlag,
    TLightGroupResetAndTurnOn,
    TLightGroupResetAndTurnOff,
    TLightGroupRestartNotifyTimer,
    TLightGroupFlashWhenOn,
    TLightGroupToggleSplitIndex,
    TLightGroupStartFlasher,
    TLightGroupCountdownEnded,
    TBumperSetBmpIndex,
    TBumperIncBmpIndex,
    TBumperDecBmpIndex,
    TComponentGroupResetNotifyTimer,
    TPopupTargetDisable,
    TPopupTargetEnable,
    TBlockerDisable,
    TBlockerEnable,
    TBlockerRestartTimeout,
    TGateDisable,
    TGateEnable,
    TKickoutRestartTimer,
    TSinkUnknown7,
    TSinkResetTimer,
    TSoloTargetDisable,
    TSoloTargetEnable,
    TTimerResetTimer,
    ControlBallCaptured,
    ControlBallReleased,
    ControlTimerExpired,
    ControlNotifyTimerExpired,
    ControlSpinnerLoopReset,
    ControlCollision,
    ControlEnableMultiplier,
    ControlDisableMultiplier,
    ControlMissionComplete,
    ControlMissionStarted,
    LeftFlipperInputPressed,
    LeftFlipperInputReleased,
    RightFlipperInputPressed,
    RightFlipperInputReleased,
    PlungerInputPressed,
    PlungerInputReleased,
    Pause,
    Resume,
    LooseFocus,
    SetTiltLock,
    ClearTiltLock,
    StartGamePlayer1,
    NewGame,
    PlungerFeedBall,
    PlungerStartFeedTimer,
    PlungerLaunchBall,
    PlungerRelaunchBall,
    PlayerChanged,
    SwitchToNextPlayer,
    GameOver,
    Reset,
}

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
    Code(MessageCode, f32),
}

impl TableMessage {
    pub const fn from_code(code: MessageCode) -> Self {
        Self::Code(code, 0.0)
    }

    pub const fn with_value(code: MessageCode, value: f32) -> Self {
        Self::Code(code, value)
    }

    pub const fn code(self) -> Option<MessageCode> {
        match self {
            Self::LeftFlipperPressed => Some(MessageCode::LeftFlipperInputPressed),
            Self::LeftFlipperReleased => Some(MessageCode::LeftFlipperInputReleased),
            Self::RightFlipperPressed => Some(MessageCode::RightFlipperInputPressed),
            Self::RightFlipperReleased => Some(MessageCode::RightFlipperInputReleased),
            Self::PlungerPressed => Some(MessageCode::PlungerInputPressed),
            Self::PlungerReleased => Some(MessageCode::PlungerInputReleased),
            Self::StartGame => Some(MessageCode::StartGamePlayer1),
            Self::Pause => Some(MessageCode::Pause),
            Self::Resume => Some(MessageCode::Resume),
            Self::Code(code, _) => Some(code),
            Self::Nudge(_) => None,
        }
    }

    pub const fn value(self) -> f32 {
        match self {
            Self::Code(_, value) => value,
            _ => 0.0,
        }
    }

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
