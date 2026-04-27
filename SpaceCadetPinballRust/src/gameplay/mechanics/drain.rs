use crate::gameplay::components::{
    ComponentId, ComponentState, GameplayComponent, SimulationState, TableInputState, TableMessage,
};

pub struct DrainMechanic {
    state: ComponentState,
    drain_y: f32,
}

impl DrainMechanic {
    pub fn new(id: ComponentId, name: &'static str) -> Self {
        Self::from_state(ComponentState::new(id, name).with_control("BallDrainControl"))
    }

    pub fn from_state(state: ComponentState) -> Self {
        Self {
            state,
            drain_y: 408.0,
        }
    }
}

impl GameplayComponent for DrainMechanic {
    fn state(&self) -> &ComponentState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ComponentState {
        &mut self.state
    }

    fn on_message(
        &mut self,
        _message: TableMessage,
        _simulation: &mut SimulationState,
        _table_state: &TableInputState,
    ) {
    }

    fn tick(&mut self, simulation: &mut SimulationState, _table_state: &TableInputState, _dt: f32) {
        let should_drain = simulation
            .ball
            .as_ref()
            .is_some_and(|ball| ball.is_drained(self.drain_y));
        if should_drain {
            simulation.ball = None;
            simulation.drain_count = simulation.drain_count.saturating_add(1);
        }
    }
}
