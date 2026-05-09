use crate::gameplay::components::{
    ComponentId, ComponentState, GameplayComponent, MessageCode, SimulationState, TableInputState,
    TableMessage,
};
use crate::engine::physics::{CollisionContact, CollisionEdgeRole};

pub struct RolloverMechanic {
    state: ComponentState,
    rollover_flag: bool,
    rearm_timer_remaining: Option<f32>,
}

impl RolloverMechanic {
    const SHIP_REFUELED_TEXT: &str = "Ship Re-Fueled";

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

    fn fuel_rollover_rule(&self) -> Option<(i32, &'static str)> {
        match self.state.control_name {
            Some("FuelRollover1Control") => Some((1, "literoll179")),
            Some("FuelRollover2Control") => Some((3, "literoll180")),
            Some("FuelRollover3Control") => Some((5, "literoll181")),
            Some("FuelRollover4Control") => Some((7, "literoll182")),
            Some("FuelRollover5Control") => Some((9, "literoll183")),
            Some("FuelRollover6Control") => Some((11, "literoll184")),
            _ => None,
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
        simulation: &mut SimulationState,
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

                if let Some((threshold, indicator_light_name)) = self.fuel_rollover_rule() {
                    if simulation.fuel_bargraph_index() > threshold {
                        simulation.queue_component_message(
                            indicator_light_name,
                            TableMessage::with_value(MessageCode::TLightTurnOffTimed, 0.05),
                        );
                    } else {
                        simulation.queue_component_message(
                            "fuel_bargraph",
                            TableMessage::with_value(
                                MessageCode::TLightGroupToggleSplitIndex,
                                threshold as f32,
                            ),
                        );
                        simulation.display_info_text(Self::SHIP_REFUELED_TEXT, 2.0);
                    }
                    simulation.add_score(self.state.collision_score());
                }
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

    fn on_collision(
        &mut self,
        _slot: u8,
        _edge_role: CollisionEdgeRole,
        _contact: CollisionContact,
        simulation: &mut SimulationState,
        table_state: &TableInputState,
    ) {
        if simulation.tilt_locked {
            return;
        }
        self.on_message(
            TableMessage::from_code(MessageCode::ControlCollision),
            simulation,
            table_state,
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::physics::{CollisionContact, CollisionEdgeRole};
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

    #[test]
    fn rollover_collision_hook_respects_tilt_lock() {
        let mut rollover = RolloverMechanic::new(ComponentId(1), "a_roll1");
        let mut simulation = SimulationState::default();
        let table_state = TableInputState::default();
        simulation.tilt_locked = true;

        rollover.on_collision(
            0,
            CollisionEdgeRole::Solid,
            CollisionContact::new(crate::engine::math::Vec2::ZERO, crate::engine::math::Vec2::ZERO, 0.0),
            &mut simulation,
            &table_state,
        );
        assert_eq!(rollover.state.sprite_index, 0);
        assert!(!rollover.rollover_flag);
    }

    #[test]
    fn fuel_rollover_refuels_bargraph_when_below_threshold() {
        let mut rollover = RolloverMechanic::from_state(
            ComponentState::new(ComponentId(1), "a_roll179")
                .with_control("FuelRollover1Control")
                .with_scoring([500]),
        );
        let mut simulation = SimulationState::default();
        let table_state = TableInputState::default();

        rollover.on_collision(
            0,
            CollisionEdgeRole::Solid,
            CollisionContact::new(crate::engine::math::Vec2::ZERO, crate::engine::math::Vec2::ZERO, 0.0),
            &mut simulation,
            &table_state,
        );

        assert_eq!(simulation.score(), 500);
        assert_eq!(simulation.info_text(), Some("Ship Re-Fueled"));
        assert_eq!(
            simulation.drain_pending_component_messages(),
            vec![(
                "fuel_bargraph".to_string(),
                TableMessage::with_value(MessageCode::TLightGroupToggleSplitIndex, 1.0),
            )]
        );
    }

    #[test]
    fn fuel_rollover_flashes_indicator_when_bargraph_already_ahead() {
        let mut rollover = RolloverMechanic::from_state(
            ComponentState::new(ComponentId(1), "a_roll180")
                .with_control("FuelRollover2Control")
                .with_scoring([500]),
        );
        let mut simulation = SimulationState::default();
        simulation.set_fuel_bargraph_index(4);
        let table_state = TableInputState::default();

        rollover.on_collision(
            0,
            CollisionEdgeRole::Solid,
            CollisionContact::new(crate::engine::math::Vec2::ZERO, crate::engine::math::Vec2::ZERO, 0.0),
            &mut simulation,
            &table_state,
        );

        assert_eq!(simulation.score(), 500);
        assert_eq!(
            simulation.drain_pending_component_messages(),
            vec![(
                "literoll180".to_string(),
                TableMessage::with_value(MessageCode::TLightTurnOffTimed, 0.05),
            )]
        );
    }
}
