use crate::gameplay::components::{
    ComponentId, ComponentState, GameplayComponent, MessageCode, SimulationState, TableInputState,
    TableMessage,
};

pub struct WallMechanic {
    state: ComponentState,
    timer_remaining: Option<f32>,
    timer_time: f32,
}

impl WallMechanic {
    pub fn new(id: ComponentId, name: &'static str) -> Self {
        Self::from_state(ComponentState::new(id, name))
    }

    pub fn from_state(mut state: ComponentState) -> Self {
        state.active = true;
        state.sprite_index = -1;
        Self {
            state,
            timer_remaining: None,
            timer_time: 0.1,
        }
    }
}

impl GameplayComponent for WallMechanic {
    fn state(&self) -> &ComponentState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ComponentState {
        &mut self.state
    }

    fn on_message(
        &mut self,
        message: TableMessage,
        _simulation: &mut SimulationState,
        _table_state: &TableInputState,
    ) {
        match message {
            TableMessage::Code(MessageCode::Reset, _) => {
                self.timer_remaining = None;
                self.state.sprite_index = -1;
                self.state.message_field = 0;
            }
            TableMessage::Code(MessageCode::ControlCollision, _) => {
                self.state.sprite_index = 0;
                self.state.message_field = 1;
                self.timer_remaining = Some(self.timer_time);
            }
            _ => {}
        }
    }

    fn on_collision(
        &mut self,
        _slot: u8,
        edge_role: crate::engine::physics::CollisionEdgeRole,
        contact: crate::engine::physics::CollisionContact,
        simulation: &mut SimulationState,
        table_state: &TableInputState,
    ) {
        if edge_role == crate::engine::physics::CollisionEdgeRole::Solid && contact.threshold_exceeded
        {
            simulation.add_score(self.state.collision_score());
        }
        self.on_message(
            TableMessage::from_code(MessageCode::ControlCollision),
            simulation,
            table_state,
        );
    }

    fn tick(&mut self, _simulation: &mut SimulationState, _table_state: &TableInputState, dt: f32) {
        if let Some(timer_remaining) = self.timer_remaining.as_mut() {
            *timer_remaining -= dt.max(0.0);
            if *timer_remaining <= 0.0 {
                self.timer_remaining = None;
                self.state.sprite_index = -1;
                self.state.message_field = 0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::gameplay::components::TableMessage;

    use super::*;

    #[test]
    fn wall_collision_flashes_then_resets() {
        let mut wall = WallMechanic::new(ComponentId(1), "v_rebo1");
        let mut simulation = SimulationState::default();
        let table_state = TableInputState::default();

        wall.on_message(
            TableMessage::from_code(MessageCode::ControlCollision),
            &mut simulation,
            &table_state,
        );
        assert_eq!(wall.state.sprite_index, 0);
        assert_eq!(wall.state.message_field, 1);

        wall.tick(&mut simulation, &table_state, 0.1);
        assert_eq!(wall.state.sprite_index, -1);
        assert_eq!(wall.state.message_field, 0);
    }

    #[test]
    fn wall_collision_adds_score() {
        let mut wall =
            WallMechanic::from_state(ComponentState::new(ComponentId(1), "v_rebo1").with_scoring([500]));
        let mut simulation = SimulationState::default();
        let table_state = TableInputState::default();

        wall.on_collision(
            0,
            crate::engine::physics::CollisionEdgeRole::Solid,
            crate::engine::physics::CollisionContact {
                point: crate::engine::math::Vec2::ZERO,
                normal: crate::engine::math::Vec2::ZERO,
                distance: 0.0,
                impact_speed: 10.0,
                threshold_exceeded: true,
                owner_token: None,
                edge_role: crate::engine::physics::CollisionEdgeRole::Solid,
            },
            &mut simulation,
            &table_state,
        );

        assert_eq!(simulation.score(), 500);
    }
}
