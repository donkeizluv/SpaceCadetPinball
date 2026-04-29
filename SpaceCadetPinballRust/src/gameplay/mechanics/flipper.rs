use crate::gameplay::components::{
    ComponentId, ComponentState, GameplayComponent, MessageCode, SimulationState, TableInputState,
    TableMessage,
};

pub struct FlipperMechanic {
    state: ComponentState,
    left_active: bool,
    right_active: bool,
    motion_active: bool,
    motion_progress: f32,
    sprite_frames: i32,
}

impl FlipperMechanic {
    pub fn new(id: ComponentId, name: &'static str) -> Self {
        Self::from_state(ComponentState::new(id, name).with_control("FlipperControl"))
    }

    pub fn from_state(state: ComponentState) -> Self {
        Self {
            state,
            left_active: false,
            right_active: false,
            motion_active: false,
            motion_progress: 0.0,
            sprite_frames: 8,
        }
    }

    fn set_motion(&mut self, extending: bool, simulation: &mut SimulationState) {
        self.motion_active = extending;
        self.state.message_field = i32::from(extending);
        if extending {
            self.motion_progress = 1.0;
            self.state.sprite_index = self.sprite_frames.saturating_sub(1);
        } else {
            self.motion_progress = 0.0;
            self.state.sprite_index = 0;
        }
        simulation.left_flipper_active = self.left_active && self.motion_active;
        simulation.right_flipper_active = self.right_active && self.motion_active;
    }
}

impl GameplayComponent for FlipperMechanic {
    fn state(&self) -> &ComponentState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ComponentState {
        &mut self.state
    }

    fn on_message(
        &mut self,
        message: TableMessage,
        simulation: &mut SimulationState,
        _table_state: &TableInputState,
    ) {
        match message {
            TableMessage::LeftFlipperPressed
            | TableMessage::Code(MessageCode::LeftFlipperInputPressed, _) => {
                self.left_active = true;
                self.set_motion(true, simulation);
            }
            TableMessage::LeftFlipperReleased
            | TableMessage::Code(MessageCode::LeftFlipperInputReleased, _) => {
                self.left_active = false;
                self.set_motion(false, simulation);
            }
            TableMessage::RightFlipperPressed
            | TableMessage::Code(MessageCode::RightFlipperInputPressed, _) => {
                self.right_active = true;
                self.set_motion(true, simulation);
            }
            TableMessage::RightFlipperReleased
            | TableMessage::Code(MessageCode::RightFlipperInputReleased, _) => {
                self.right_active = false;
                self.set_motion(false, simulation);
            }
            TableMessage::Code(MessageCode::TFlipperExtend, _) => {
                self.left_active = true;
                self.right_active = true;
                self.set_motion(true, simulation);
            }
            TableMessage::Code(
                MessageCode::TFlipperRetract
                | MessageCode::Resume
                | MessageCode::LooseFocus
                | MessageCode::SetTiltLock
                | MessageCode::GameOver,
                _,
            ) => {
                self.set_motion(false, simulation);
            }
            TableMessage::Code(MessageCode::PlayerChanged | MessageCode::Reset, _) => {
                self.left_active = false;
                self.right_active = false;
                self.motion_active = false;
                self.motion_progress = 0.0;
                self.state.message_field = 0;
                self.state.sprite_index = 0;
                simulation.left_flipper_active = false;
                simulation.right_flipper_active = false;
            }
            _ => {}
        }
    }

    fn tick(&mut self, simulation: &mut SimulationState, _table_state: &TableInputState, _dt: f32) {
        if let Some(ball) = simulation.active_ball_mut() {
            ball.apply_flipper_impulse(self.left_active, self.right_active);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::gameplay::components::TableMessage;

    use super::*;

    #[test]
    fn flipper_extend_and_reset_follow_message_state() {
        let mut flipper = FlipperMechanic::new(ComponentId(1), "flipper");
        let mut simulation = SimulationState::default();
        let table_state = TableInputState::default();

        flipper.on_message(
            TableMessage::from_code(MessageCode::TFlipperExtend),
            &mut simulation,
            &table_state,
        );
        assert_eq!(flipper.state.message_field, 1);
        assert!(simulation.left_flipper_active);
        assert!(simulation.right_flipper_active);

        flipper.on_message(
            TableMessage::from_code(MessageCode::Reset),
            &mut simulation,
            &table_state,
        );
        assert_eq!(flipper.state.message_field, 0);
        assert_eq!(flipper.state.sprite_index, 0);
        assert!(!simulation.left_flipper_active);
        assert!(!simulation.right_flipper_active);
    }
}
