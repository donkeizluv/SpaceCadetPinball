use crate::gameplay::components::{
    ComponentId, ComponentState, GameplayComponent, MessageCode, SimulationState, TableInputState,
    TableMessage,
};

const PLAYER_COUNT: usize = 4;

#[derive(Debug, Clone, Copy)]
struct PlayerLightState {
    flasher_on: bool,
    light_on_bmp_index: i32,
    light_on: bool,
    message_field: i32,
}

impl Default for PlayerLightState {
    fn default() -> Self {
        Self {
            flasher_on: false,
            light_on_bmp_index: 0,
            light_on: false,
            message_field: 0,
        }
    }
}

pub struct LightMechanic {
    state: ComponentState,
    current_player: usize,
    player_data: [PlayerLightState; PLAYER_COUNT],
    timeout_remaining: Option<f32>,
    undo_override_remaining: Option<f32>,
    flash_remaining: Option<f32>,
    flash_delay: [f32; 2],
    source_delay: [f32; 2],
    light_on: bool,
    flasher_on: bool,
    flash_light_on: bool,
    toggled_off: bool,
    toggled_on: bool,
    temporary_override: bool,
    turn_off_after_flashing: bool,
    previous_bitmap: i32,
    bmp_arr: [i32; 2],
    light_on_bmp_index: i32,
}

impl LightMechanic {
    pub fn new(id: ComponentId, name: &'static str) -> Self {
        Self::from_state(ComponentState::new(id, name).with_control("LightControl"))
    }

    pub fn from_state(state: ComponentState) -> Self {
        let mut light = Self {
            state,
            current_player: 0,
            player_data: [PlayerLightState::default(); PLAYER_COUNT],
            timeout_remaining: None,
            undo_override_remaining: None,
            flash_remaining: None,
            flash_delay: [0.1, 0.1],
            source_delay: [0.1, 0.1],
            light_on: false,
            flasher_on: false,
            flash_light_on: false,
            toggled_off: false,
            toggled_on: false,
            temporary_override: false,
            turn_off_after_flashing: false,
            previous_bitmap: -1,
            bmp_arr: [-1, 0],
            light_on_bmp_index: 0,
        };
        light.reset();
        light
    }

    fn reset(&mut self) {
        self.timeout_remaining = None;
        self.undo_override_remaining = None;
        self.flash_remaining = None;
        self.light_on = false;
        self.light_on_bmp_index = 0;
        self.toggled_off = false;
        self.toggled_on = false;
        self.flasher_on = false;
        self.flash_light_on = false;
        self.temporary_override = false;
        self.turn_off_after_flashing = false;
        self.previous_bitmap = -1;
        self.bmp_arr = [-1, 0];
        self.state.sprite_index = self.bmp_arr[0];
        self.state.message_field = 0;
    }

    fn schedule_timeout(&mut self, time: f32) {
        self.flash_delay = self.source_delay;
        self.timeout_remaining = (time > 0.0).then_some(time);
    }

    fn current_sprite_index(&self) -> i32 {
        if self.flasher_on {
            self.bmp_arr[self.flash_light_on as usize]
        } else if self.toggled_off {
            self.bmp_arr[0]
        } else if self.toggled_on || self.light_on {
            self.bmp_arr[1]
        } else {
            self.bmp_arr[0]
        }
    }

    fn set_sprite_bmp(&mut self, index: i32) {
        self.previous_bitmap = index;
        if !self.temporary_override {
            self.state.sprite_index = index;
        }
    }

    fn flasher_start(&mut self, bmp_index: bool) {
        self.flash_light_on = bmp_index;
        self.set_sprite_bmp(self.bmp_arr[self.flash_light_on as usize]);
        self.flash_remaining = Some(self.flash_delay[self.flash_light_on as usize]);
    }

    fn flasher_stop(&mut self, bmp_index: Option<bool>) {
        self.flash_remaining = None;
        if let Some(bmp_index) = bmp_index {
            self.flash_light_on = bmp_index;
            self.set_sprite_bmp(self.bmp_arr[self.flash_light_on as usize]);
        }
    }
}

impl GameplayComponent for LightMechanic {
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
                self.reset();
                for player_state in &mut self.player_data {
                    player_state.flasher_on = self.flasher_on;
                    player_state.light_on_bmp_index = self.light_on_bmp_index;
                    player_state.light_on = self.light_on;
                    player_state.message_field = self.state.message_field;
                }
            }
            TableMessage::Code(MessageCode::PlayerChanged, value) => {
                let next_player = value.floor().clamp(0.0, (PLAYER_COUNT - 1) as f32) as usize;
                let player_state = &mut self.player_data[self.current_player];
                player_state.flasher_on = self.flasher_on;
                player_state.light_on_bmp_index = self.light_on_bmp_index;
                player_state.light_on = self.light_on;
                player_state.message_field = self.state.message_field;

                self.reset();

                let player_state = self.player_data[next_player];
                self.current_player = next_player;
                self.flasher_on = player_state.flasher_on;
                self.light_on_bmp_index = player_state.light_on_bmp_index;
                self.light_on = player_state.light_on;
                self.state.message_field = player_state.message_field;
                if self.light_on_bmp_index != 0 {
                    self.on_message(
                        TableMessage::with_value(
                            MessageCode::TLightSetOnStateBmpIndex,
                            self.light_on_bmp_index as f32,
                        ),
                        _simulation,
                        _table_state,
                    );
                }
                if self.light_on {
                    self.on_message(TableMessage::from_code(MessageCode::TLightTurnOn), _simulation, _table_state);
                }
                if self.flasher_on {
                    self.on_message(
                        TableMessage::from_code(MessageCode::TLightFlasherStart),
                        _simulation,
                        _table_state,
                    );
                }
            }
            TableMessage::Code(MessageCode::TLightTurnOff, _) => {
                self.light_on = false;
                if !self.flasher_on && !self.toggled_off && !self.toggled_on {
                    self.set_sprite_bmp(self.bmp_arr[0]);
                }
            }
            TableMessage::Code(MessageCode::TLightTurnOn, _) => {
                self.light_on = true;
                if !self.flasher_on && !self.toggled_off && !self.toggled_on {
                    self.set_sprite_bmp(self.bmp_arr[1]);
                }
            }
            TableMessage::Code(MessageCode::TLightFlasherStart, _) => {
                self.schedule_timeout(0.0);
                if !self.flasher_on || self.flash_remaining.is_none() {
                    self.flasher_on = true;
                    self.toggled_on = false;
                    self.toggled_off = false;
                    self.turn_off_after_flashing = false;
                    self.flasher_start(self.light_on);
                }
            }
            TableMessage::Code(MessageCode::TLightApplyMultDelay, value) => {
                self.flash_delay[0] = value * self.source_delay[0];
                self.flash_delay[1] = value * self.source_delay[1];
            }
            TableMessage::Code(MessageCode::TLightApplyDelay, _) => {
                self.flash_delay = self.source_delay;
            }
            TableMessage::Code(MessageCode::TLightFlasherStartTimed, value) => {
                if !self.flasher_on {
                    self.flasher_start(self.light_on);
                }
                self.flasher_on = true;
                self.toggled_on = false;
                self.turn_off_after_flashing = false;
                self.toggled_off = false;
                self.schedule_timeout(value);
            }
            TableMessage::Code(MessageCode::TLightTurnOffTimed, value) => {
                if !self.toggled_off {
                    if self.flasher_on {
                        self.flasher_stop(Some(false));
                        self.flasher_on = false;
                    } else {
                        self.set_sprite_bmp(self.bmp_arr[0]);
                    }
                    self.toggled_off = true;
                    self.toggled_on = false;
                }
                self.schedule_timeout(value);
            }
            TableMessage::Code(MessageCode::TLightTurnOnTimed, value) => {
                if !self.toggled_on {
                    if self.flasher_on {
                        self.flasher_stop(Some(true));
                        self.flasher_on = false;
                    } else {
                        self.set_sprite_bmp(self.bmp_arr[1]);
                    }
                    self.toggled_on = true;
                    self.toggled_off = false;
                }
                self.schedule_timeout(value);
            }
            TableMessage::Code(MessageCode::TLightSetOnStateBmpIndex, value) => {
                self.light_on_bmp_index = value.floor().max(0.0) as i32;
                self.bmp_arr = [-1, self.light_on_bmp_index];
                self.set_sprite_bmp(self.current_sprite_index());
            }
            TableMessage::Code(MessageCode::TLightIncOnStateBmpIndex, _) => {
                self.on_message(
                    TableMessage::with_value(
                        MessageCode::TLightSetOnStateBmpIndex,
                        (self.light_on_bmp_index + 1) as f32,
                    ),
                    _simulation,
                    _table_state,
                );
            }
            TableMessage::Code(MessageCode::TLightDecOnStateBmpIndex, _) => {
                self.on_message(
                    TableMessage::with_value(
                        MessageCode::TLightSetOnStateBmpIndex,
                        (self.light_on_bmp_index - 1).max(0) as f32,
                    ),
                    _simulation,
                    _table_state,
                );
            }
            TableMessage::Code(MessageCode::TLightResetTimed, _) => {
                self.timeout_remaining = None;
                if self.flasher_on {
                    self.flasher_stop(None);
                }
                self.flasher_on = false;
                self.toggled_off = false;
                self.toggled_on = false;
                self.set_sprite_bmp(self.bmp_arr[self.light_on as usize]);
            }
            TableMessage::Code(MessageCode::TLightFlasherStartTimedThenStayOn, value) => {
                self.turn_off_after_flashing = false;
                self.undo_override_remaining = None;
                self.on_message(TableMessage::from_code(MessageCode::TLightTurnOn), _simulation, _table_state);
                self.on_message(
                    TableMessage::with_value(MessageCode::TLightFlasherStartTimed, value),
                    _simulation,
                    _table_state,
                );
            }
            TableMessage::Code(MessageCode::TLightFlasherStartTimedThenStayOff, value) => {
                self.undo_override_remaining = None;
                self.on_message(
                    TableMessage::with_value(MessageCode::TLightFlasherStartTimed, value),
                    _simulation,
                    _table_state,
                );
                self.turn_off_after_flashing = true;
            }
            TableMessage::Code(MessageCode::TLightToggleValue, value) => {
                let code = if value.floor() != 0.0 {
                    MessageCode::TLightTurnOn
                } else {
                    MessageCode::TLightTurnOff
                };
                self.on_message(TableMessage::from_code(code), _simulation, _table_state);
            }
            TableMessage::Code(MessageCode::TLightResetAndToggleValue, value) => {
                self.on_message(
                    TableMessage::with_value(MessageCode::TLightToggleValue, value),
                    _simulation,
                    _table_state,
                );
                self.on_message(TableMessage::from_code(MessageCode::TLightResetTimed), _simulation, _table_state);
            }
            TableMessage::Code(MessageCode::TLightResetAndTurnOn, _) => {
                self.on_message(TableMessage::from_code(MessageCode::TLightTurnOn), _simulation, _table_state);
                self.on_message(TableMessage::from_code(MessageCode::TLightResetTimed), _simulation, _table_state);
            }
            TableMessage::Code(MessageCode::TLightResetAndTurnOff, _) => {
                self.on_message(TableMessage::from_code(MessageCode::TLightTurnOff), _simulation, _table_state);
                self.on_message(TableMessage::from_code(MessageCode::TLightResetTimed), _simulation, _table_state);
            }
            TableMessage::Code(MessageCode::TLightToggle, _) => {
                let value = if self.light_on { 0.0 } else { 1.0 };
                self.on_message(
                    TableMessage::with_value(MessageCode::TLightResetAndToggleValue, value),
                    _simulation,
                    _table_state,
                );
            }
            TableMessage::Code(MessageCode::TLightResetAndToggle, _) => {
                let value = if self.light_on { 0.0 } else { 1.0 };
                self.on_message(
                    TableMessage::with_value(MessageCode::TLightResetAndToggleValue, value),
                    _simulation,
                    _table_state,
                );
            }
            TableMessage::Code(MessageCode::TLightSetMessageField, value) => {
                self.state.message_field = value.floor() as i32;
            }
            TableMessage::Code(MessageCode::TLightFtTmpOverrideOn | MessageCode::TLightFtTmpOverrideOff, value) => {
                let sprite_index = if matches!(
                    message,
                    TableMessage::Code(MessageCode::TLightFtTmpOverrideOn, _)
                ) {
                    self.bmp_arr[1]
                } else {
                    self.bmp_arr[0]
                };
                self.state.sprite_index = sprite_index;
                self.undo_override_remaining = None;
                if value > 0.0 {
                    self.temporary_override = true;
                    self.undo_override_remaining = Some(value);
                }
            }
            TableMessage::Code(MessageCode::TLightFtResetOverride, _) => {
                self.undo_override_remaining = None;
                self.temporary_override = false;
                self.state.sprite_index = self.previous_bitmap;
            }
            _ => {}
        }
    }

    fn tick(&mut self, _simulation: &mut SimulationState, _table_state: &TableInputState, dt: f32) {
        let dt = dt.max(0.0);

        if let Some(timeout_remaining) = self.timeout_remaining.as_mut() {
            *timeout_remaining -= dt;
            if *timeout_remaining <= 0.0 {
                self.timeout_remaining = None;
                if self.flasher_on {
                    self.flasher_stop(None);
                }
                self.set_sprite_bmp(self.bmp_arr[self.light_on as usize]);
                self.toggled_off = false;
                self.toggled_on = false;
                self.flasher_on = false;
                if self.turn_off_after_flashing {
                    self.turn_off_after_flashing = false;
                    self.on_message(
                        TableMessage::from_code(MessageCode::TLightResetAndTurnOff),
                        _simulation,
                        _table_state,
                    );
                }
            }
        }

        if let Some(undo_override_remaining) = self.undo_override_remaining.as_mut() {
            *undo_override_remaining -= dt;
            if *undo_override_remaining <= 0.0 {
                self.on_message(
                    TableMessage::from_code(MessageCode::TLightFtResetOverride),
                    _simulation,
                    _table_state,
                );
            }
        }

        if let Some(flash_remaining) = self.flash_remaining.as_mut() {
            *flash_remaining -= dt;
            if *flash_remaining <= 0.0 {
                self.flash_light_on = !self.flash_light_on;
                self.set_sprite_bmp(self.bmp_arr[self.flash_light_on as usize]);
                self.flash_remaining = Some(self.flash_delay[self.flash_light_on as usize]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::gameplay::components::TableMessage;

    use super::*;

    #[test]
    fn light_turn_on_and_timed_reset_restore_sprite_state() {
        let mut light = LightMechanic::new(ComponentId(1), "test_light");
        let mut simulation = SimulationState::default();
        let table_state = TableInputState::default();

        light.on_message(
            TableMessage::from_code(MessageCode::TLightTurnOn),
            &mut simulation,
            &table_state,
        );
        assert_eq!(light.state.sprite_index, 0);

        light.on_message(
            TableMessage::with_value(MessageCode::TLightTurnOffTimed, 0.25),
            &mut simulation,
            &table_state,
        );
        assert_eq!(light.state.sprite_index, -1);

        light.tick(&mut simulation, &table_state, 0.25);
        assert_eq!(light.state.sprite_index, 0);
    }
}
