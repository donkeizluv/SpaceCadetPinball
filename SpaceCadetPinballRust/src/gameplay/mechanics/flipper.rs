use crate::gameplay::components::{
    ComponentId, GameplayComponent, SimulationState, TableInputState, TableMessage,
};

pub struct FlipperMechanic {
    id: ComponentId,
    name: &'static str,
    left_active: bool,
    right_active: bool,
}

impl FlipperMechanic {
    pub fn new(id: ComponentId, name: &'static str) -> Self {
        Self {
            id,
            name,
            left_active: false,
            right_active: false,
        }
    }
}

impl GameplayComponent for FlipperMechanic {
    fn id(&self) -> ComponentId {
        self.id
    }

    fn name(&self) -> &str {
        self.name
    }

    fn on_message(
        &mut self,
        message: TableMessage,
        simulation: &mut SimulationState,
        _table_state: &TableInputState,
    ) {
        match message {
            TableMessage::LeftFlipperPressed => {
                self.left_active = true;
                simulation.left_flipper_active = true;
            }
            TableMessage::LeftFlipperReleased => {
                self.left_active = false;
                simulation.left_flipper_active = false;
            }
            TableMessage::RightFlipperPressed => {
                self.right_active = true;
                simulation.right_flipper_active = true;
            }
            TableMessage::RightFlipperReleased => {
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
