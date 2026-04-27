use crate::gameplay::components::{
    ComponentId, ComponentState, GameplayComponent, MessageCode, SimulationState, TableInputState,
    TableMessage,
};

pub struct LightRolloverMechanic {
    state: ComponentState,
    rollover_flag: bool,
    rearm_timer_remaining: Option<f32>,
    clear_timer_remaining: Option<f32>,
    clear_delay: f32,
}

impl LightRolloverMechanic {
    pub fn new(id: ComponentId, name: &'static str) -> Self {
        Self::from_state(ComponentState::new(id, name).with_control("LightRolloverControl"))
    }

    pub fn from_state(mut state: ComponentState) -> Self {
        state.active = true;
        state.sprite_index = -1;
        Self {
            state,
            rollover_flag: false,
            rearm_timer_remaining: None,
            clear_timer_remaining: None,
            clear_delay: 0.5,
        }
    }
}

impl GameplayComponent for LightRolloverMechanic {
    fn state(&self) -> &ComponentState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ComponentState {
        &mut self.state
    }

    fn collision_edge_active(&self, slot: u8) -> bool {
        match slot {
            0 => self.state.active,
            1 => self.rollover_flag,
            _ => false,
        }
    }

    fn on_message(
        &mut self,
        message: TableMessage,
        _simulation: &mut SimulationState,
        _table_state: &TableInputState,
    ) {
        if let TableMessage::Code(code, _) = message {
            match code {
                MessageCode::Reset => {
                    self.state.active = true;
                    self.rollover_flag = false;
                    self.rearm_timer_remaining = None;
                    self.clear_timer_remaining = None;
                    self.state.sprite_index = -1;
                }
                MessageCode::ControlCollision => {
                    if self.rollover_flag {
                        self.state.active = false;
                        self.rollover_flag = false;
                        self.rearm_timer_remaining = Some(0.1);
                        if self.clear_timer_remaining.is_none() {
                            self.clear_timer_remaining = Some(self.clear_delay);
                        }
                    } else {
                        self.rollover_flag = true;
                        self.state.sprite_index = 0;
                    }
                }
                _ => {}
            }
        }
    }

    fn tick(&mut self, _simulation: &mut SimulationState, _table_state: &TableInputState, dt: f32) {
        let dt = dt.max(0.0);

        if let Some(rearm_timer_remaining) = self.rearm_timer_remaining.as_mut() {
            *rearm_timer_remaining -= dt;
            if *rearm_timer_remaining <= 0.0 {
                self.rearm_timer_remaining = None;
                self.state.active = true;
            }
        }

        if let Some(clear_timer_remaining) = self.clear_timer_remaining.as_mut() {
            *clear_timer_remaining -= dt;
            if *clear_timer_remaining <= 0.0 {
                self.clear_timer_remaining = None;
                self.state.sprite_index = -1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::gameplay::components::{GameplayComponent, TableMessage};

    use super::*;

    #[test]
    fn light_rollover_holds_light_then_clears_after_delay() {
        let mut rollover = LightRolloverMechanic::new(ComponentId(1), "a_roll9");
        let mut simulation = SimulationState::default();
        let table_state = TableInputState::default();

        rollover.on_message(
            TableMessage::from_code(MessageCode::ControlCollision),
            &mut simulation,
            &table_state,
        );
        assert_eq!(rollover.state.sprite_index, 0);

        rollover.on_message(
            TableMessage::from_code(MessageCode::ControlCollision),
            &mut simulation,
            &table_state,
        );
        assert!(!rollover.state.active);
        assert_eq!(rollover.state.sprite_index, 0);

        rollover.tick(&mut simulation, &table_state, 0.1);
        assert!(rollover.state.active);
        assert_eq!(rollover.state.sprite_index, 0);

        rollover.tick(&mut simulation, &table_state, 0.5);
        assert_eq!(rollover.state.sprite_index, -1);
    }

    #[test]
    fn light_rollover_secondary_wall_tracks_rollover_flag() {
        let mut rollover = LightRolloverMechanic::new(ComponentId(1), "a_roll9");
        let mut simulation = SimulationState::default();
        let table_state = TableInputState::default();

        assert!(rollover.collision_edge_active(0));
        assert!(!rollover.collision_edge_active(1));

        rollover.on_message(
            TableMessage::from_code(MessageCode::ControlCollision),
            &mut simulation,
            &table_state,
        );
        assert!(rollover.collision_edge_active(1));
    }
}
