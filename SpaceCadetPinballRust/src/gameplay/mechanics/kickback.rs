use crate::gameplay::components::{
    ComponentId, ComponentState, GameplayComponent, MessageCode, SimulationState, TableInputState,
    TableMessage,
};

pub struct KickbackMechanic {
    state: ComponentState,
    timer_remaining: Option<f32>,
    kick_active: bool,
    post_kick_window: bool,
    first_stage_duration: f32,
    second_stage_duration: f32,
}

impl KickbackMechanic {
    pub fn new(id: ComponentId, name: &'static str) -> Self {
        Self::from_state(ComponentState::new(id, name).with_control("KickbackControl"))
    }

    pub fn from_state(mut state: ComponentState) -> Self {
        state.sprite_index = -1;
        Self {
            state,
            timer_remaining: None,
            kick_active: false,
            post_kick_window: false,
            first_stage_duration: 0.7,
            second_stage_duration: 0.1,
        }
    }

    fn reset_cycle(&mut self) {
        self.timer_remaining = None;
        self.kick_active = false;
        self.post_kick_window = false;
        self.state.sprite_index = -1;
    }
}

impl GameplayComponent for KickbackMechanic {
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
            TableMessage::Code(MessageCode::ControlCollision, _) => {
                if self.timer_remaining.is_none() {
                    self.kick_active = true;
                    self.post_kick_window = false;
                    self.timer_remaining = Some(self.first_stage_duration);
                }
            }
            TableMessage::Code(MessageCode::SetTiltLock | MessageCode::Reset, _) => {
                self.reset_cycle();
            }
            _ => {}
        }
    }

    fn tick(&mut self, _simulation: &mut SimulationState, _table_state: &TableInputState, dt: f32) {
        let Some(timer_remaining) = self.timer_remaining.as_mut() else {
            return;
        };

        *timer_remaining -= dt.max(0.0);
        if *timer_remaining > 0.0 {
            return;
        }

        if self.kick_active {
            self.kick_active = false;
            self.post_kick_window = true;
            self.state.sprite_index = 1;
            self.timer_remaining = Some(self.second_stage_duration);
        } else if self.post_kick_window {
            self.post_kick_window = false;
            self.state.sprite_index = 0;
            self.timer_remaining = None;
        } else {
            self.timer_remaining = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::gameplay::components::TableMessage;

    use super::*;

    #[test]
    fn kickback_collision_runs_two_stage_timer_animation() {
        let mut kickback = KickbackMechanic::new(ComponentId(1), "a_kick1");
        let mut simulation = SimulationState::default();
        let table_state = TableInputState::default();

        kickback.on_message(
            TableMessage::from_code(MessageCode::ControlCollision),
            &mut simulation,
            &table_state,
        );
        assert_eq!(kickback.timer_remaining, Some(0.7));
        assert_eq!(kickback.state.sprite_index, -1);

        kickback.tick(&mut simulation, &table_state, 0.7);
        assert_eq!(kickback.state.sprite_index, 1);
        assert_eq!(kickback.timer_remaining, Some(0.1));

        kickback.tick(&mut simulation, &table_state, 0.1);
        assert_eq!(kickback.state.sprite_index, 0);
        assert_eq!(kickback.timer_remaining, None);
    }
}
