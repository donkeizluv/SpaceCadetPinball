use crate::gameplay::components::{
    ComponentId, ComponentState, GameplayComponent, MessageCode, SimulationState, TableInputState,
    TableMessage,
};

const PLAYER_COUNT: usize = 4;

pub struct SinkMechanic {
    state: ComponentState,
    current_player: usize,
    player_message_field_backup: [i32; PLAYER_COUNT],
    reset_timer_remaining: Option<f32>,
    default_reset_time: f32,
}

impl SinkMechanic {
    pub fn new(id: ComponentId, name: &'static str) -> Self {
        Self::from_state(ComponentState::new(id, name).with_control("SinkControl"))
    }

    pub fn from_state(state: ComponentState) -> Self {
        Self {
            state,
            current_player: 0,
            player_message_field_backup: [0; PLAYER_COUNT],
            reset_timer_remaining: None,
            default_reset_time: 0.5,
        }
    }
}

impl GameplayComponent for SinkMechanic {
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
            TableMessage::Code(MessageCode::TSinkResetTimer, value) => {
                self.reset_timer_remaining =
                    Some(if value < 0.0 { self.default_reset_time } else { value });
            }
            TableMessage::Code(MessageCode::PlayerChanged, value) => {
                let next_player = value.floor().clamp(0.0, (PLAYER_COUNT - 1) as f32) as usize;
                self.player_message_field_backup[self.current_player] = self.state.message_field;
                self.state.message_field = self.player_message_field_backup[next_player];
                self.current_player = next_player;
            }
            TableMessage::Code(MessageCode::Reset, _) => {
                self.reset_timer_remaining = None;
                self.current_player = 0;
                self.state.message_field = 0;
                self.player_message_field_backup.fill(0);
            }
            _ => {}
        }
    }

    fn tick(&mut self, _simulation: &mut SimulationState, _table_state: &TableInputState, dt: f32) {
        if let Some(reset_timer_remaining) = self.reset_timer_remaining.as_mut() {
            *reset_timer_remaining -= dt.max(0.0);
            if *reset_timer_remaining <= 0.0 {
                self.reset_timer_remaining = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::gameplay::components::TableMessage;

    use super::*;

    #[test]
    fn sink_preserves_message_field_per_player() {
        let mut sink = SinkMechanic::new(ComponentId(1), "v_sink1");
        let mut simulation = SimulationState::default();
        let table_state = TableInputState::default();

        sink.state.message_field = 7;
        sink.on_message(
            TableMessage::with_value(MessageCode::PlayerChanged, 1.0),
            &mut simulation,
            &table_state,
        );
        assert_eq!(sink.state.message_field, 0);

        sink.state.message_field = 13;
        sink.on_message(
            TableMessage::with_value(MessageCode::PlayerChanged, 0.0),
            &mut simulation,
            &table_state,
        );
        assert_eq!(sink.state.message_field, 7);
    }
}
