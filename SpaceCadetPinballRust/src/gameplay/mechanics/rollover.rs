use crate::gameplay::components::{
    ComponentId, ComponentState, GameplayComponent, MessageCode, SimulationState, TableInputState,
    TableMessage,
};

pub struct RolloverMechanic {
    state: ComponentState,
    rollover_flag: bool,
    rearm_timer_remaining: Option<f32>,
}

impl RolloverMechanic {
    pub fn new(id: ComponentId, name: &'static str) -> Self {
        Self::from_state(ComponentState::new(id, name).with_control("RolloverControl"))
    }

    pub fn from_state(mut state: ComponentState) -> Self {
        state.active = true;
        state.sprite_index = 0;
        Self {
            state,
            rollover_flag: false,
            rearm_timer_remaining: None,
        }
    }
}

impl GameplayComponent for RolloverMechanic {
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
        match message {
            TableMessage::Code(MessageCode::Reset, _) => {
                self.state.active = true;
                self.rollover_flag = false;
                self.rearm_timer_remaining = None;
                self.state.sprite_index = 0;
            }
            TableMessage::Code(MessageCode::ControlCollision, _) => {
                if self.rollover_flag {
                    self.state.active = false;
                    self.rearm_timer_remaining = Some(0.1);
                }
                self.rollover_flag = !self.rollover_flag;
                self.state.sprite_index = if self.rollover_flag { -1 } else { 0 };
            }
            _ => {}
        }
    }

    fn tick(&mut self, _simulation: &mut SimulationState, _table_state: &TableInputState, dt: f32) {
        if let Some(rearm_timer_remaining) = self.rearm_timer_remaining.as_mut() {
            *rearm_timer_remaining -= dt.max(0.0);
            if *rearm_timer_remaining <= 0.0 {
                self.rearm_timer_remaining = None;
                self.state.active = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::gameplay::components::{GameplayComponent, TableMessage};

    use super::*;

    #[test]
    fn rollover_collision_toggles_sprite_and_rearms_after_second_hit() {
        let mut rollover = RolloverMechanic::new(ComponentId(1), "a_roll1");
        let mut simulation = SimulationState::default();
        let table_state = TableInputState::default();

        rollover.on_message(
            TableMessage::from_code(MessageCode::ControlCollision),
            &mut simulation,
            &table_state,
        );
        assert_eq!(rollover.state.sprite_index, -1);
        assert!(rollover.state.active);

        rollover.on_message(
            TableMessage::from_code(MessageCode::ControlCollision),
            &mut simulation,
            &table_state,
        );
        assert_eq!(rollover.state.sprite_index, 0);
        assert!(!rollover.state.active);

        rollover.tick(&mut simulation, &table_state, 0.1);
        assert!(rollover.state.active);
    }

    #[test]
    fn rollover_secondary_wall_tracks_rollover_flag() {
        let mut rollover = RolloverMechanic::new(ComponentId(1), "a_roll1");
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
