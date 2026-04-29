use crate::gameplay::components::{
    ComponentId, ComponentState, GameplayComponent, MessageCode, SimulationState, TableInputState,
    TableMessage,
};

pub struct DrainMechanic {
    state: ComponentState,
    drain_y: f32,
    timer_remaining: Option<f32>,
    timer_time: f32,
}

impl DrainMechanic {
    pub fn new(id: ComponentId, name: &'static str) -> Self {
        Self::from_state(ComponentState::new(id, name).with_control("BallDrainControl"))
    }

    pub fn from_state(state: ComponentState) -> Self {
        Self {
            state,
            drain_y: 408.0,
            timer_remaining: None,
            timer_time: 1.0,
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

    fn collision_edge_offset(&self, slot: u8, collision_component_offset: f32) -> f32 {
        if slot == 0 {
            collision_component_offset
        } else {
            0.0
        }
    }

    fn on_message(
        &mut self,
        message: TableMessage,
        _simulation: &mut SimulationState,
        _table_state: &TableInputState,
    ) {
        if let TableMessage::Code(MessageCode::Reset, _) = message {
            self.timer_remaining = None;
        }
    }

    fn tick(&mut self, simulation: &mut SimulationState, _table_state: &TableInputState, dt: f32) {
        let drained_count = simulation.remove_drained_balls(self.drain_y);
        if drained_count > 0 {
            simulation.drain_count = simulation.drain_count.saturating_add(drained_count as u64);
            if simulation.multiball_count == 0 {
                simulation.ball_in_drain = true;
                self.timer_remaining = Some(self.timer_time);
            }
        }

        if let Some(timer) = self.timer_remaining.as_mut() {
            *timer -= dt.max(0.0);
            if *timer <= 0.0 {
                self.timer_remaining = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::math::Vec2;
    use crate::gameplay::components::TableMessage;

    use super::*;

    #[test]
    fn drain_sets_ball_in_drain_and_timer_when_last_ball_drains() {
        let mut drain = DrainMechanic::new(ComponentId(1), "drain");
        let mut simulation = SimulationState::default();
        let table_state = TableInputState::default();
        let _ = simulation.add_ball(Vec2::new(100.0, 420.0));

        drain.tick(&mut simulation, &table_state, 0.0);
        assert!(!simulation.has_active_ball());
        assert!(simulation.ball_in_drain);
        assert_eq!(simulation.multiball_count, 0);
        assert_eq!(simulation.drain_count, 1);
        assert_eq!(drain.timer_remaining, Some(1.0));

        drain.on_message(
            TableMessage::from_code(MessageCode::Reset),
            &mut simulation,
            &table_state,
        );
        assert_eq!(drain.timer_remaining, None);
    }
}
