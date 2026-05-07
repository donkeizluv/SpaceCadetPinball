use crate::engine::physics::{CollisionContact, CollisionEdgeRole};
use crate::gameplay::components::{
    ComponentId, ComponentState, GameplayComponent, MessageCode, SimulationState, TableInputState,
    TableMessage,
};

const PLAYER_COUNT: usize = 4;

pub struct PopupTargetMechanic {
    state: ComponentState,
    timer_remaining: Option<f32>,
    timer_time: f32,
    player_message_field_backup: [i32; PLAYER_COUNT],
    current_player: usize,
}

impl PopupTargetMechanic {
    pub fn new(id: ComponentId, name: &'static str) -> Self {
        Self::from_state(ComponentState::new(id, name))
    }

    pub fn from_state(mut state: ComponentState) -> Self {
        state.active = true;
        state.sprite_index = 0;
        Self {
            state,
            timer_remaining: None,
            timer_time: 0.1,
            player_message_field_backup: [0; PLAYER_COUNT],
            current_player: 0,
        }
    }

    fn disable(&mut self) {
        self.state.active = false;
        self.state.sprite_index = -1;
    }

    fn schedule_enable(&mut self) {
        self.timer_remaining = Some(self.timer_time);
    }

    fn enable_now(&mut self) {
        self.timer_remaining = None;
        self.state.active = true;
        self.state.sprite_index = 0;
    }
}

impl GameplayComponent for PopupTargetMechanic {
    fn state(&self) -> &ComponentState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ComponentState {
        &mut self.state
    }

    fn apply_float_attribute(&mut self, attribute_id: i16, values: &[f32]) {
        if attribute_id == 407
            && let Some(value) = values.first().copied()
            && value > 0.0
        {
            self.timer_time = value;
        }
    }

    fn on_message(
        &mut self,
        message: TableMessage,
        _simulation: &mut SimulationState,
        _table_state: &TableInputState,
    ) {
        match message {
            TableMessage::Code(MessageCode::TPopupTargetDisable, _) => {
                self.disable();
            }
            TableMessage::Code(MessageCode::TPopupTargetEnable, _) => {
                self.schedule_enable();
            }
            TableMessage::Code(MessageCode::PlayerChanged, value) => {
                let next_player = value.floor().clamp(0.0, (PLAYER_COUNT - 1) as f32) as usize;
                self.player_message_field_backup[self.current_player] = self.state.message_field;
                self.current_player = next_player;
                self.state.message_field = self.player_message_field_backup[next_player];
                if self.state.message_field != 0 {
                    self.disable();
                } else {
                    self.schedule_enable();
                }
            }
            TableMessage::Code(MessageCode::Reset, _) => {
                self.state.message_field = 0;
                self.player_message_field_backup = [0; PLAYER_COUNT];
                self.current_player = 0;
                self.enable_now();
            }
            _ => {}
        }
    }

    fn on_collision(
        &mut self,
        _slot: u8,
        edge_role: CollisionEdgeRole,
        contact: CollisionContact,
        simulation: &mut SimulationState,
        table_state: &TableInputState,
    ) {
        if simulation.tilt_locked
            || edge_role != CollisionEdgeRole::Solid
            || !contact.threshold_exceeded
            || !self.state.active
        {
            return;
        }

        self.disable();
        simulation.add_score(self.state.collision_score());
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
                self.enable_now();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::math::Vec2;

    use super::*;

    #[test]
    fn popup_target_disables_on_hard_hit() {
        let mut target = PopupTargetMechanic::new(ComponentId(1), "a_targ1");
        let mut simulation = SimulationState::default();
        let table_state = TableInputState::default();

        target.on_collision(
            0,
            CollisionEdgeRole::Solid,
            CollisionContact {
                point: Vec2::ZERO,
                normal: Vec2::new(0.0, -1.0),
                distance: 0.0,
                impact_speed: 12.0,
                threshold_exceeded: true,
                owner_token: None,
                edge_role: CollisionEdgeRole::Solid,
            },
            &mut simulation,
            &table_state,
        );

        assert!(!target.state.active);
        assert_eq!(target.state.sprite_index, -1);
    }

    #[test]
    fn popup_target_player_changed_restores_saved_message_field_state() {
        let mut target = PopupTargetMechanic::new(ComponentId(1), "a_targ1");
        let mut simulation = SimulationState::default();
        let table_state = TableInputState::default();

        target.state.message_field = 1;
        target.disable();
        target.on_message(
            TableMessage::with_value(MessageCode::PlayerChanged, 1.0),
            &mut simulation,
            &table_state,
        );
        assert_eq!(target.state.message_field, 0);
        assert!(!target.state.active);
        target.tick(&mut simulation, &table_state, 0.1);
        assert!(target.state.active);

        target.on_message(
            TableMessage::with_value(MessageCode::PlayerChanged, 0.0),
            &mut simulation,
            &table_state,
        );
        assert_eq!(target.state.message_field, 1);
        assert!(!target.state.active);
    }

    #[test]
    fn popup_target_uses_dat_timer_attribute_when_provided() {
        let mut target = PopupTargetMechanic::new(ComponentId(1), "a_targ1");
        target.apply_float_attribute(407, &[0.35]);

        let mut simulation = SimulationState::default();
        let table_state = TableInputState::default();
        target.disable();
        target.on_message(
            TableMessage::from_code(MessageCode::TPopupTargetEnable),
            &mut simulation,
            &table_state,
        );

        target.tick(&mut simulation, &table_state, 0.34);
        assert!(!target.state.active);
        target.tick(&mut simulation, &table_state, 0.01);
        assert!(target.state.active);
    }

    #[test]
    fn popup_target_hard_hit_adds_score() {
        let mut target =
            PopupTargetMechanic::from_state(ComponentState::new(ComponentId(1), "a_targ1").with_scoring([500, 5000]));
        let mut simulation = SimulationState::default();
        let table_state = TableInputState::default();

        target.on_collision(
            0,
            CollisionEdgeRole::Solid,
            CollisionContact {
                point: Vec2::ZERO,
                normal: Vec2::new(0.0, -1.0),
                distance: 0.0,
                impact_speed: 12.0,
                threshold_exceeded: true,
                owner_token: None,
                edge_role: CollisionEdgeRole::Solid,
            },
            &mut simulation,
            &table_state,
        );

        assert_eq!(simulation.score(), 500);
    }
}
