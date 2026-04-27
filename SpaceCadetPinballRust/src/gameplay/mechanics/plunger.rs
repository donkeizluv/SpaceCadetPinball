use crate::engine::physics::Ball;
use crate::gameplay::components::{
    ComponentId, ComponentState, GameplayComponent, MessageCode, SimulationState, TableInputState,
    TableMessage,
};

pub struct PlungerMechanic {
    state: ComponentState,
    charging: bool,
}

impl PlungerMechanic {
    pub fn new(id: ComponentId, name: &'static str) -> Self {
        Self::from_state(ComponentState::new(id, name).with_control("PlungerControl"))
    }

    pub fn from_state(state: ComponentState) -> Self {
        Self {
            state,
            charging: false,
        }
    }
}

impl GameplayComponent for PlungerMechanic {
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
            TableMessage::StartGame
            | TableMessage::Code(MessageCode::StartGamePlayer1, _)
            | TableMessage::Code(MessageCode::NewGame, _)
            | TableMessage::Code(MessageCode::PlungerFeedBall, _) => {
                if simulation.ball.is_none() {
                    simulation.ball = Some(Ball::ready_in_launch_lane());
                }
            }
            TableMessage::PlungerPressed
            | TableMessage::Code(MessageCode::PlungerInputPressed, _) => {
                self.charging = true;
            }
            TableMessage::PlungerReleased
            | TableMessage::Code(MessageCode::PlungerInputReleased, _)
            | TableMessage::Code(MessageCode::PlungerLaunchBall, _) => {
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
