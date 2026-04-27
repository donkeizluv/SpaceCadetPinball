use crate::gameplay::components::{
    ComponentId, ComponentState, GameplayComponent, MessageCode, SimulationState, TableInputState,
    TableMessage,
};

pub struct TripwireMechanic {
    state: ComponentState,
    trigger_count: u32,
}

impl TripwireMechanic {
    pub fn new(id: ComponentId, name: &'static str) -> Self {
        Self::from_state(ComponentState::new(id, name).with_control("TripwireControl"))
    }

    pub fn from_state(mut state: ComponentState) -> Self {
        state.active = true;
        state.sprite_index = 0;
        Self {
            state,
            trigger_count: 0,
        }
    }
}

impl GameplayComponent for TripwireMechanic {
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
                self.state.active = true;
                self.state.sprite_index = 0;
                self.trigger_count = 0;
            }
            TableMessage::Code(MessageCode::ControlCollision, _) => {
                self.trigger_count = self.trigger_count.saturating_add(1);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::gameplay::components::TableMessage;

    use super::*;

    #[test]
    fn tripwire_counts_triggers_and_resets() {
        let mut tripwire = TripwireMechanic::new(ComponentId(1), "s_trip1");
        let mut simulation = SimulationState::default();
        let table_state = TableInputState::default();

        tripwire.on_message(
            TableMessage::from_code(MessageCode::ControlCollision),
            &mut simulation,
            &table_state,
        );
        tripwire.on_message(
            TableMessage::from_code(MessageCode::ControlCollision),
            &mut simulation,
            &table_state,
        );
        assert_eq!(tripwire.trigger_count, 2);

        tripwire.on_message(
            TableMessage::from_code(MessageCode::Reset),
            &mut simulation,
            &table_state,
        );
        assert_eq!(tripwire.trigger_count, 0);
        assert!(tripwire.state.active);
    }
}
