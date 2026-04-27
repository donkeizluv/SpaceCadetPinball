use crate::gameplay::components::{
    ComponentId, ComponentState, GameplayComponent, MessageCode, SimulationState, TableInputState,
    TableMessage,
};

pub struct FlipperMechanic {
    state: ComponentState,
    left_active: bool,
    right_active: bool,
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
        }
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
                simulation.left_flipper_active = true;
            }
            TableMessage::LeftFlipperReleased
            | TableMessage::Code(MessageCode::LeftFlipperInputReleased, _) => {
                self.left_active = false;
                simulation.left_flipper_active = false;
            }
            TableMessage::RightFlipperPressed
            | TableMessage::Code(MessageCode::RightFlipperInputPressed, _) => {
                self.right_active = true;
                simulation.right_flipper_active = true;
            }
            TableMessage::RightFlipperReleased
            | TableMessage::Code(MessageCode::RightFlipperInputReleased, _) => {
                self.right_active = false;
                simulation.right_flipper_active = false;
            }
            _ => {}
        }
    }

    fn tick(&mut self, simulation: &mut SimulationState, _table_state: &TableInputState, _dt: f32) {
        if let Some(ball) = simulation.ball.as_mut() {
            ball.apply_flipper_impulse(self.left_active, self.right_active);
        }
    }
}
