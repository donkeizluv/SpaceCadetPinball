use crate::gameplay::components::{
    ComponentId, ComponentState, GameplayComponent, MessageCode, SimulationState, TableInputState,
    TableMessage,
};

pub struct BlockerMechanic {
    state: ComponentState,
    timeout_remaining: Option<f32>,
}

impl BlockerMechanic {
    pub fn new(id: ComponentId, name: &'static str) -> Self {
        Self::from_state(ComponentState::new(id, name).with_control("DrainBallBlockerControl"))
    }

    pub fn from_state(mut state: ComponentState) -> Self {
        state.active = false;
        state.sprite_index = -1;
        Self {
            state,
            timeout_remaining: None,
        }
    }
}

impl GameplayComponent for BlockerMechanic {
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
        match message {
            TableMessage::Code(
                MessageCode::SetTiltLock
                | MessageCode::PlayerChanged
                | MessageCode::Reset
                | MessageCode::TBlockerDisable,
                _,
            ) => {
                self.timeout_remaining = None;
                self.state.message_field = 0;
                self.state.active = false;
                self.state.sprite_index = -1;
            }
            TableMessage::Code(MessageCode::TBlockerEnable, value) => {
                self.state.active = true;
                self.state.sprite_index = 0;
                self.timeout_remaining = (value >= 0.0).then_some(value);
            }
            TableMessage::Code(MessageCode::TBlockerRestartTimeout, value) => {
                self.timeout_remaining = Some(value.max(0.0));
            }
            _ => {}
        }
    }

    fn tick(&mut self, _simulation: &mut SimulationState, _table_state: &TableInputState, dt: f32) {
        if let Some(timeout_remaining) = self.timeout_remaining.as_mut() {
            *timeout_remaining -= dt.max(0.0);
            if *timeout_remaining <= 0.0 {
                self.timeout_remaining = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::gameplay::components::TableMessage;

    use super::*;

    #[test]
    fn blocker_enable_disable_tracks_active_sprite_and_timer() {
        let mut blocker = BlockerMechanic::new(ComponentId(1), "v_bloc1");
        let mut simulation = SimulationState::default();
        let table_state = TableInputState::default();

        blocker.on_message(
            TableMessage::with_value(MessageCode::TBlockerEnable, 1.25),
            &mut simulation,
            &table_state,
        );
        assert!(blocker.state.active);
        assert_eq!(blocker.state.sprite_index, 0);
        assert_eq!(blocker.timeout_remaining, Some(1.25));

        blocker.tick(&mut simulation, &table_state, 1.5);
        assert_eq!(blocker.timeout_remaining, None);

        blocker.on_message(
            TableMessage::from_code(MessageCode::TBlockerDisable),
            &mut simulation,
            &table_state,
        );
        assert!(!blocker.state.active);
        assert_eq!(blocker.state.sprite_index, -1);
        assert_eq!(blocker.state.message_field, 0);
    }
}
