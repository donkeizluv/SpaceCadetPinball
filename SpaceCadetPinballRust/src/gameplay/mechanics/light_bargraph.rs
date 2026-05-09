use crate::gameplay::components::{
    ComponentId, ComponentState, GameplayComponent, MessageCode, SimulationState, TableInputState,
    TableMessage,
};

const PLAYER_COUNT: usize = 4;
const DEFAULT_LIGHT_COUNT: u8 = 6;

pub struct LightBargraphMechanic {
    state: ComponentState,
    current_player: usize,
    light_count: u8,
    time_index: i32,
    timer_remaining: Option<f32>,
    timer_time_array: Vec<f32>,
    player_time_index_backup: [i32; PLAYER_COUNT],
}

impl LightBargraphMechanic {
    pub fn new(id: ComponentId, name: &'static str) -> Self {
        Self::from_state(ComponentState::new(id, name), DEFAULT_LIGHT_COUNT)
    }

    pub fn from_state(state: ComponentState, light_count: u8) -> Self {
        Self {
            state,
            current_player: 0,
            light_count: light_count.max(1),
            time_index: 0,
            timer_remaining: None,
            timer_time_array: Vec::new(),
            player_time_index_backup: [0; PLAYER_COUNT],
        }
    }

    fn max_time_index(&self) -> i32 {
        i32::from(self.light_count) * 2 - 1
    }

    fn reset_runtime_state(&mut self, simulation: &mut SimulationState) {
        self.time_index = 0;
        self.timer_remaining = None;
        self.state.message_field = 0;
        self.state.sprite_index = -1;
        simulation.set_fuel_bargraph_index(0);
    }

    fn apply_time_index(&mut self, raw_index: i32, simulation: &mut SimulationState) {
        self.timer_remaining = None;

        if raw_index < 0 {
            self.reset_runtime_state(simulation);
            self.player_time_index_backup[self.current_player] = self.time_index;
            return;
        }

        self.time_index = raw_index.clamp(0, self.max_time_index());
        self.state.message_field = self.time_index;
        self.state.sprite_index = self.time_index / 2;
        self.player_time_index_backup[self.current_player] = self.time_index;
        simulation.set_fuel_bargraph_index(self.time_index);

        if let Some(duration) = self.timer_time_array.get(self.time_index as usize).copied()
            && duration > 0.0
        {
            self.timer_remaining = Some(duration);
        }
    }
}

impl GameplayComponent for LightBargraphMechanic {
    fn state(&self) -> &ComponentState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ComponentState {
        &mut self.state
    }

    fn apply_float_attribute(&mut self, attribute_id: i16, values: &[f32]) {
        if attribute_id == 904 && !values.is_empty() {
            self.timer_time_array = values.to_vec();
            self.light_count = (values.len() / 2).max(1) as u8;
        }
    }

    fn on_message(
        &mut self,
        message: TableMessage,
        simulation: &mut SimulationState,
        _table_state: &TableInputState,
    ) {
        match message {
            TableMessage::Code(MessageCode::TLightGroupToggleSplitIndex, value) => {
                self.apply_time_index(value.floor() as i32, simulation);
            }
            TableMessage::Code(MessageCode::TLightResetAndTurnOff | MessageCode::Reset, _) => {
                self.current_player = 0;
                self.player_time_index_backup = [0; PLAYER_COUNT];
                self.reset_runtime_state(simulation);
            }
            TableMessage::Code(MessageCode::SetTiltLock, _) => {
                self.reset_runtime_state(simulation);
                self.player_time_index_backup[self.current_player] = 0;
            }
            TableMessage::Code(MessageCode::PlayerChanged, value) => {
                let next_player = value.floor().clamp(0.0, (PLAYER_COUNT - 1) as f32) as usize;
                self.player_time_index_backup[self.current_player] = self.time_index;
                self.reset_runtime_state(simulation);
                self.current_player = next_player;
                let restored_index = self.player_time_index_backup[next_player];
                if restored_index > 0 {
                    self.apply_time_index(restored_index, simulation);
                }
            }
            _ => {}
        }
    }

    fn tick(&mut self, simulation: &mut SimulationState, _table_state: &TableInputState, dt: f32) {
        if let Some(timer_remaining) = self.timer_remaining.as_mut() {
            *timer_remaining -= dt.max(0.0);
            if *timer_remaining <= 0.0 {
                self.timer_remaining = None;
                if self.time_index > 0 {
                    self.apply_time_index(self.time_index - 1, simulation);
                    simulation.queue_message(TableMessage::from_code(MessageCode::ControlTimerExpired));
                } else {
                    self.reset_runtime_state(simulation);
                    self.player_time_index_backup[self.current_player] = 0;
                    simulation.queue_message(TableMessage::from_code(
                        MessageCode::TLightGroupCountdownEnded,
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_split_index_uses_dat_timer_schedule_and_counts_down() {
        let mut bargraph = LightBargraphMechanic::new(ComponentId(1), "fuel_bargraph");
        bargraph.apply_float_attribute(904, &[0.5, 0.4, 0.3, 0.2]);
        let mut simulation = SimulationState::default();
        let table_state = TableInputState::default();

        bargraph.on_message(
            TableMessage::with_value(MessageCode::TLightGroupToggleSplitIndex, 3.0),
            &mut simulation,
            &table_state,
        );
        assert_eq!(bargraph.state.message_field, 3);
        assert_eq!(bargraph.state.sprite_index, 1);

        bargraph.tick(&mut simulation, &table_state, 0.2);
        assert_eq!(bargraph.state.message_field, 2);
        assert_eq!(
            simulation.drain_pending_messages(),
            vec![TableMessage::from_code(MessageCode::ControlTimerExpired)]
        );
    }

    #[test]
    fn countdown_end_queues_light_group_countdown_message() {
        let mut bargraph = LightBargraphMechanic::new(ComponentId(1), "fuel_bargraph");
        bargraph.apply_float_attribute(904, &[0.1, 0.1]);
        let mut simulation = SimulationState::default();
        let table_state = TableInputState::default();

        bargraph.on_message(
            TableMessage::with_value(MessageCode::TLightGroupToggleSplitIndex, 0.0),
            &mut simulation,
            &table_state,
        );
        bargraph.tick(&mut simulation, &table_state, 0.1);

        assert_eq!(bargraph.state.message_field, 0);
        assert_eq!(bargraph.state.sprite_index, -1);
        assert_eq!(
            simulation.drain_pending_messages(),
            vec![TableMessage::from_code(MessageCode::TLightGroupCountdownEnded)]
        );
    }

    #[test]
    fn player_change_restores_saved_time_index() {
        let mut bargraph = LightBargraphMechanic::new(ComponentId(1), "fuel_bargraph");
        bargraph.apply_float_attribute(904, &[0.1, 0.1, 0.1, 0.1]);
        let mut simulation = SimulationState::default();
        let table_state = TableInputState::default();

        bargraph.on_message(
            TableMessage::with_value(MessageCode::TLightGroupToggleSplitIndex, 3.0),
            &mut simulation,
            &table_state,
        );
        bargraph.on_message(
            TableMessage::with_value(MessageCode::PlayerChanged, 1.0),
            &mut simulation,
            &table_state,
        );
        assert_eq!(bargraph.state.message_field, 0);

        bargraph.on_message(
            TableMessage::with_value(MessageCode::PlayerChanged, 0.0),
            &mut simulation,
            &table_state,
        );
        assert_eq!(bargraph.state.message_field, 3);
        assert_eq!(bargraph.state.sprite_index, 1);
    }
}
