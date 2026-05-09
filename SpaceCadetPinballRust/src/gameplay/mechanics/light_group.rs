use crate::gameplay::components::{
    ComponentId, ComponentState, GameplayComponent, MessageCode, SimulationState, TableInputState,
    TableMessage,
};

const PLAYER_COUNT: usize = 4;

pub struct LightGroupMechanic {
    state: ComponentState,
    current_player: usize,
    light_count: u8,
    active_count: u8,
    notify_timer_remaining: Option<f32>,
    player_active_count_backup: [u8; PLAYER_COUNT],
}

impl LightGroupMechanic {
    pub fn new(id: ComponentId, name: &'static str, light_count: u8) -> Self {
        Self::from_state(ComponentState::new(id, name), light_count)
    }

    pub fn from_state(state: ComponentState, light_count: u8) -> Self {
        Self {
            state,
            current_player: 0,
            light_count: light_count.max(1),
            active_count: 0,
            notify_timer_remaining: None,
            player_active_count_backup: [0; PLAYER_COUNT],
        }
    }

    fn set_active_count(&mut self, active_count: u8) {
        self.active_count = active_count.min(self.light_count);
        self.state.message_field = i32::from(self.active_count);
        self.player_active_count_backup[self.current_player] = self.active_count;
    }

    fn restart_notify_timer(&mut self, duration_seconds: f32) {
        self.notify_timer_remaining = (duration_seconds > 0.0).then_some(duration_seconds);
    }

    fn on_notify_timer_expired(&mut self, simulation: &mut SimulationState) {
        if self.state.control_name == Some("MultiplierLightGroupControl") && simulation.score_multiplier > 0 {
            simulation.score_multiplier = simulation.score_multiplier.saturating_sub(1);
        }

        self.set_active_count(self.active_count.saturating_sub(1));
        if self.active_count > 0 {
            self.restart_notify_timer(30.0);
        }
    }
}

impl GameplayComponent for LightGroupMechanic {
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
            TableMessage::Code(MessageCode::PlayerChanged, value) => {
                let next_player = value.floor().clamp(0.0, (PLAYER_COUNT - 1) as f32) as usize;
                self.player_active_count_backup[self.current_player] = self.active_count;
                self.current_player = next_player;
                self.set_active_count(self.player_active_count_backup[next_player]);
            }
            TableMessage::Code(MessageCode::Reset, _) => {
                self.current_player = 0;
                self.notify_timer_remaining = None;
                self.player_active_count_backup = [0; PLAYER_COUNT];
                self.set_active_count(0);
            }
            TableMessage::Code(MessageCode::TLightGroupResetAndTurnOn, _) => {
                self.set_active_count(self.active_count.saturating_add(1));
            }
            TableMessage::Code(MessageCode::TLightFlasherStartTimedThenStayOff, _) => {
                self.notify_timer_remaining = None;
                self.set_active_count(0);
            }
            TableMessage::Code(MessageCode::TLightResetAndTurnOn, _) => {
                self.set_active_count(self.light_count);
            }
            TableMessage::Code(MessageCode::TLightResetAndTurnOff, _) => {
                self.notify_timer_remaining = None;
                self.set_active_count(0);
            }
            TableMessage::Code(MessageCode::TLightGroupOffsetAnimationBackward, _) => {
                self.set_active_count(self.active_count.saturating_sub(1));
            }
            TableMessage::Code(MessageCode::TLightGroupRestartNotifyTimer, value) => {
                self.restart_notify_timer(value);
            }
            TableMessage::Code(MessageCode::ControlEnableMultiplier, _) => {
                simulation.score_multiplier = 4;
                self.set_active_count(self.light_count);
                self.restart_notify_timer(30.0);
                simulation.display_info_text("10X MULTIPLIER", 2.0);
            }
            TableMessage::Code(MessageCode::ControlDisableMultiplier, _) => {
                simulation.score_multiplier = 0;
                self.notify_timer_remaining = None;
                self.set_active_count(0);
            }
            _ => {}
        }
    }

    fn tick(&mut self, simulation: &mut SimulationState, _table_state: &TableInputState, dt: f32) {
        if let Some(remaining) = self.notify_timer_remaining.as_mut() {
            *remaining -= dt.max(0.0);
            if *remaining <= 0.0 {
                self.notify_timer_remaining = None;
                self.on_notify_timer_expired(simulation);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multiplier_light_group_enable_sets_full_multiplier_and_timer() {
        let mut group = LightGroupMechanic::new(ComponentId(1), "top_target_lights", 4);
        group.state.control_name = Some("MultiplierLightGroupControl");
        let mut simulation = SimulationState::default();
        let table_state = TableInputState::default();

        group.on_message(
            TableMessage::from_code(MessageCode::ControlEnableMultiplier),
            &mut simulation,
            &table_state,
        );

        assert_eq!(group.state.message_field, 4);
        assert_eq!(simulation.score_multiplier, 4);
        assert_eq!(simulation.info_text(), Some("10X MULTIPLIER"));
    }

    #[test]
    fn multiplier_light_group_notify_expiry_steps_back_multiplier() {
        let mut group = LightGroupMechanic::new(ComponentId(1), "top_target_lights", 4);
        group.state.control_name = Some("MultiplierLightGroupControl");
        let mut simulation = SimulationState::default();
        let table_state = TableInputState::default();

        group.on_message(
            TableMessage::from_code(MessageCode::ControlEnableMultiplier),
            &mut simulation,
            &table_state,
        );
        group.tick(&mut simulation, &table_state, 30.0);

        assert_eq!(group.state.message_field, 3);
        assert_eq!(simulation.score_multiplier, 3);
    }

    #[test]
    fn medal_light_group_tracks_per_player_active_count() {
        let mut group = LightGroupMechanic::new(ComponentId(1), "bumper_target_lights", 3);
        group.state.control_name = Some("MedalLightGroupControl");
        let mut simulation = SimulationState::default();
        let table_state = TableInputState::default();

        group.on_message(
            TableMessage::from_code(MessageCode::TLightGroupResetAndTurnOn),
            &mut simulation,
            &table_state,
        );
        group.on_message(
            TableMessage::with_value(MessageCode::PlayerChanged, 1.0),
            &mut simulation,
            &table_state,
        );
        assert_eq!(group.state.message_field, 0);

        group.on_message(
            TableMessage::with_value(MessageCode::PlayerChanged, 0.0),
            &mut simulation,
            &table_state,
        );
        assert_eq!(group.state.message_field, 1);
    }

    #[test]
    fn group_flash_then_stay_off_clears_count() {
        let mut group = LightGroupMechanic::new(ComponentId(1), "top_circle_tgt_lights", 3);
        let mut simulation = SimulationState::default();
        let table_state = TableInputState::default();

        group.on_message(
            TableMessage::from_code(MessageCode::TLightGroupResetAndTurnOn),
            &mut simulation,
            &table_state,
        );
        group.on_message(
            TableMessage::from_code(MessageCode::TLightGroupResetAndTurnOn),
            &mut simulation,
            &table_state,
        );
        group.on_message(
            TableMessage::with_value(MessageCode::TLightFlasherStartTimedThenStayOff, 2.0),
            &mut simulation,
            &table_state,
        );

        assert_eq!(group.state.message_field, 0);
    }
}
