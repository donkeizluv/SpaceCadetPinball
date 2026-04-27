use crate::gameplay::components::{
    CollisionGeometryKind, ComponentId, ComponentState, GameplayComponent, MessageCode,
    SimulationState, TableInputState, TableMessage,
};

pub struct KickoutMechanic {
    state: ComponentState,
    release_timer_remaining: Option<f32>,
    reset_timer_remaining: Option<f32>,
    capture_pending: bool,
    active_after_reset: bool,
    default_release_time: f32,
    default_reset_time: f32,
}

impl KickoutMechanic {
    pub fn new(id: ComponentId, name: &'static str) -> Self {
        Self::from_state(ComponentState::new(id, name).with_control("KickoutControl"))
    }

    pub fn from_state(mut state: ComponentState) -> Self {
        state.active = true;
        Self {
            state,
            release_timer_remaining: None,
            reset_timer_remaining: None,
            capture_pending: false,
            active_after_reset: true,
            default_release_time: 1.5,
            default_reset_time: 0.05,
        }
    }

    fn release_captured_ball(&mut self) {
        self.capture_pending = false;
        self.release_timer_remaining = None;
        self.reset_timer_remaining = Some(self.default_reset_time);
        self.state.active = false;
    }
}

impl GameplayComponent for KickoutMechanic {
    fn state(&self) -> &ComponentState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ComponentState {
        &mut self.state
    }

    fn collision_geometry_kind(&self) -> CollisionGeometryKind {
        CollisionGeometryKind::VisualCircleAttribute306
    }

    fn on_message(
        &mut self,
        message: TableMessage,
        _simulation: &mut SimulationState,
        _table_state: &TableInputState,
    ) {
        match message {
            TableMessage::Code(MessageCode::ControlBallCaptured | MessageCode::ControlCollision, _) => {
                self.capture_pending = true;
                self.release_timer_remaining = None;
            }
            TableMessage::Code(MessageCode::ControlBallReleased, _) => {
                self.release_captured_ball();
            }
            TableMessage::Code(MessageCode::TKickoutRestartTimer, value) => {
                if self.capture_pending {
                    self.release_timer_remaining =
                        Some(if value < 0.0 { self.default_release_time } else { value });
                }
            }
            TableMessage::Code(MessageCode::SetTiltLock | MessageCode::Reset, _) => {
                if self.capture_pending {
                    self.release_captured_ball();
                }
                self.release_timer_remaining = None;
                if matches!(message, TableMessage::Code(MessageCode::SetTiltLock, _)) {
                    self.state.active = false;
                }
            }
            _ => {}
        }
    }

    fn tick(&mut self, _simulation: &mut SimulationState, _table_state: &TableInputState, dt: f32) {
        let dt = dt.max(0.0);
        let mut released_this_tick = false;

        if let Some(release_timer_remaining) = self.release_timer_remaining.as_mut() {
            *release_timer_remaining -= dt;
            if *release_timer_remaining <= 0.0 {
                self.release_captured_ball();
                released_this_tick = true;
            }
        }

        if released_this_tick {
            return;
        }

        if let Some(reset_timer_remaining) = self.reset_timer_remaining.as_mut() {
            *reset_timer_remaining -= dt;
            if *reset_timer_remaining <= 0.0 {
                self.reset_timer_remaining = None;
                if self.active_after_reset {
                    self.state.active = true;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::gameplay::components::TableMessage;

    use super::*;

    #[test]
    fn kickout_restart_timer_releases_captured_ball_and_rearms() {
        let mut kickout = KickoutMechanic::new(ComponentId(1), "a_kout1");
        let mut simulation = SimulationState::default();
        let table_state = TableInputState::default();

        assert!(kickout.state.active);

        kickout.on_message(
            TableMessage::from_code(MessageCode::ControlBallCaptured),
            &mut simulation,
            &table_state,
        );
        kickout.on_message(
            TableMessage::with_value(MessageCode::TKickoutRestartTimer, -1.0),
            &mut simulation,
            &table_state,
        );

        assert!(kickout.capture_pending);
        assert_eq!(kickout.release_timer_remaining, Some(1.5));

        kickout.tick(&mut simulation, &table_state, 1.5);
        assert!(!kickout.capture_pending);
        assert!(!kickout.state.active);

        kickout.tick(&mut simulation, &table_state, 0.05);
        assert!(kickout.state.active);
    }
}
