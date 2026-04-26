use crate::engine::physics::Ball;
use crate::gameplay::components::{
    ComponentId, GameplayComponent, SimulationState, TableInputState, TableMessage,
};

pub struct PlungerMechanic {
    id: ComponentId,
    name: &'static str,
    charging: bool,
}

impl PlungerMechanic {
    pub fn new(id: ComponentId, name: &'static str) -> Self {
        Self {
            id,
            name,
            charging: false,
        }
    }
}

impl GameplayComponent for PlungerMechanic {
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
            TableMessage::StartGame => {
                if simulation.ball.is_none() {
                    simulation.ball = Some(Ball::ready_in_launch_lane());
                }
            }
            TableMessage::PlungerPressed => {
                self.charging = true;
            }
            TableMessage::PlungerReleased => {
                self.charging = false;
            }
            _ => {}
        }
    }

    fn tick(&mut self, simulation: &mut SimulationState, _table_state: &TableInputState, dt: f32) {
        if self.charging {
            simulation.plunger_charge = (simulation.plunger_charge + dt * 1.4).min(1.0);
            return;
        }

        if simulation.plunger_charge > 0.0 {
            if let Some(ball) = simulation.ball.as_mut()
                && !ball.is_launched()
            {
                ball.launch(simulation.plunger_charge);
                simulation.launch_count = simulation.launch_count.saturating_add(1);
            }
            simulation.plunger_charge = 0.0;
        }
    }
}
