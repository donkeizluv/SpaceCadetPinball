use crate::engine::physics::{CollisionContact, CollisionEdgeRole};
use crate::gameplay::components::{
    ComponentId, ComponentState, GameplayComponent, MessageCode, SimulationState, TableInputState,
    TableMessage,
};

const PLAYER_COUNT: usize = 4;

pub struct SoloTargetMechanic {
    state: ComponentState,
    timer_remaining: Option<f32>,
    timer_time: f32,
    player_message_field_backup: [i32; PLAYER_COUNT],
    current_player: usize,
}

impl SoloTargetMechanic {
    const SHIP_REFUELED_TEXT: &str = "Ship Re-Fueled";

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

    fn apply_active_visual(&mut self) {
        self.state.sprite_index = if self.state.active { 0 } else { 1 };
    }

    fn source_group_mask(&self) -> Option<i32> {
        match (self.state.control_name, self.state.group_name.as_str()) {
            (Some("MissionSpotTargetControl"), "a_targ13") => Some(1),
            (Some("MissionSpotTargetControl"), "a_targ14") => Some(2),
            (Some("MissionSpotTargetControl"), "a_targ15") => Some(4),
            (Some("LeftHazardSpotTargetControl"), "a_targ16") => Some(1),
            (Some("LeftHazardSpotTargetControl"), "a_targ17") => Some(2),
            (Some("LeftHazardSpotTargetControl"), "a_targ18") => Some(4),
            (Some("RightHazardSpotTargetControl"), "a_targ19") => Some(1),
            (Some("RightHazardSpotTargetControl"), "a_targ20") => Some(2),
            (Some("RightHazardSpotTargetControl"), "a_targ21") => Some(4),
            _ => None,
        }
    }

    fn fuel_spot_target_rule(&self) -> Option<(u8, &'static str)> {
        match self.state.group_name.as_str() {
            "a_targ10" if self.state.control_name == Some("FuelSpotTargetControl") => {
                Some((0b001, "lite70"))
            }
            "a_targ11" if self.state.control_name == Some("FuelSpotTargetControl") => {
                Some((0b010, "lite71"))
            }
            "a_targ12" if self.state.control_name == Some("FuelSpotTargetControl") => {
                Some((0b100, "lite72"))
            }
            _ => None,
        }
    }

    fn mission_spot_target_rule(&self) -> Option<(i32, &'static str)> {
        match self.state.group_name.as_str() {
            "a_targ13" if self.state.control_name == Some("MissionSpotTargetControl") => {
                Some((1, "lite101"))
            }
            "a_targ14" if self.state.control_name == Some("MissionSpotTargetControl") => {
                Some((2, "lite102"))
            }
            "a_targ15" if self.state.control_name == Some("MissionSpotTargetControl") => {
                Some((4, "lite103"))
            }
            _ => None,
        }
    }

    fn left_hazard_target_rule(&self) -> Option<(u8, &'static str)> {
        match self.state.group_name.as_str() {
            "a_targ16" if self.state.control_name == Some("LeftHazardSpotTargetControl") => {
                Some((0b001, "lite104"))
            }
            "a_targ17" if self.state.control_name == Some("LeftHazardSpotTargetControl") => {
                Some((0b010, "lite105"))
            }
            "a_targ18" if self.state.control_name == Some("LeftHazardSpotTargetControl") => {
                Some((0b100, "lite106"))
            }
            _ => None,
        }
    }

    fn right_hazard_target_rule(&self) -> Option<(u8, &'static str)> {
        match self.state.group_name.as_str() {
            "a_targ19" if self.state.control_name == Some("RightHazardSpotTargetControl") => {
                Some((0b001, "lite107"))
            }
            "a_targ20" if self.state.control_name == Some("RightHazardSpotTargetControl") => {
                Some((0b010, "lite108"))
            }
            "a_targ21" if self.state.control_name == Some("RightHazardSpotTargetControl") => {
                Some((0b100, "lite109"))
            }
            _ => None,
        }
    }
}

impl GameplayComponent for SoloTargetMechanic {
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
            TableMessage::Code(MessageCode::TSoloTargetDisable, _) => {
                self.state.active = false;
                self.apply_active_visual();
            }
            TableMessage::Code(MessageCode::TSoloTargetEnable, _) => {
                self.timer_remaining = Some(self.timer_time);
            }
            TableMessage::Code(MessageCode::PlayerChanged, value) => {
                let next_player = value.floor().clamp(0.0, (PLAYER_COUNT - 1) as f32) as usize;
                self.player_message_field_backup[self.current_player] = self.state.message_field;
                self.current_player = next_player;
                self.state.message_field = self.player_message_field_backup[next_player];
            }
            TableMessage::Code(MessageCode::Reset, _) => {
                self.timer_remaining = None;
                self.state.active = true;
                self.state.message_field = 0;
                self.player_message_field_backup = [0; PLAYER_COUNT];
                self.current_player = 0;
                self.apply_active_visual();
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

        self.on_message(
            TableMessage::from_code(MessageCode::TSoloTargetDisable),
            simulation,
            table_state,
        );
        if let Some((mask, light_name)) = self.fuel_spot_target_rule() {
            simulation.queue_component_message(
                light_name,
                TableMessage::with_value(MessageCode::TLightFlasherStartTimedThenStayOn, 2.0),
            );
            simulation.queue_component_message(
                "top_circle_tgt_lights",
                TableMessage::from_code(MessageCode::TLightGroupResetAndTurnOn),
            );
            let fuel_target_mask = simulation.mark_current_player_fuel_spot_target(mask);
            simulation.add_score(self.state.collision_score());
            if fuel_target_mask == 0b111 {
                simulation.queue_component_message(
                    "top_circle_tgt_lights",
                    TableMessage::with_value(MessageCode::TLightFlasherStartTimedThenStayOff, 2.0),
                );
                simulation.queue_component_message(
                    "fuel_bargraph",
                    TableMessage::with_value(MessageCode::TLightGroupToggleSplitIndex, 11.0),
                );
                simulation.display_info_text(Self::SHIP_REFUELED_TEXT, 2.0);
                simulation.clear_current_player_fuel_spot_targets();
            }
        } else if let Some((mask, light_name)) = self.mission_spot_target_rule() {
            self.state.message_field |= mask;
            self.player_message_field_backup[self.current_player] = self.state.message_field;
            simulation.queue_component_message(
                light_name,
                TableMessage::with_value(MessageCode::TLightFlasherStartTimedThenStayOn, 2.0),
            );
            simulation.queue_component_message(
                "ramp_tgt_lights",
                TableMessage::from_code(MessageCode::TLightGroupResetAndTurnOn),
            );
            simulation.add_score(self.state.collision_score());
            if self.state.message_field == 0b111 {
                simulation.queue_component_message(
                    "ramp_tgt_lights",
                    TableMessage::with_value(MessageCode::TLightFlasherStartTimedThenStayOff, 2.0),
                );
            }
        } else if let Some((mask, light_name)) = self.left_hazard_target_rule() {
            self.state.message_field |= i32::from(mask);
            self.player_message_field_backup[self.current_player] = self.state.message_field;
            simulation.queue_component_message(
                light_name,
                TableMessage::with_value(MessageCode::TLightFlasherStartTimedThenStayOn, 2.0),
            );
            simulation.queue_component_message(
                "lchute_tgt_lights",
                TableMessage::from_code(MessageCode::TLightGroupResetAndTurnOn),
            );
            let group_mask = simulation.mark_current_player_left_hazard_target(mask);
            simulation.add_score(self.state.collision_score());
            if group_mask == 0b111 {
                simulation.queue_component_message(
                    "v_gate1",
                    TableMessage::from_code(MessageCode::TGateDisable),
                );
                simulation.queue_component_message(
                    "lchute_tgt_lights",
                    TableMessage::with_value(MessageCode::TLightFlasherStartTimedThenStayOff, 2.0),
                );
                simulation.clear_current_player_left_hazard_targets();
            }
        } else if let Some((mask, light_name)) = self.right_hazard_target_rule() {
            self.state.message_field |= i32::from(mask);
            self.player_message_field_backup[self.current_player] = self.state.message_field;
            simulation.queue_component_message(
                light_name,
                TableMessage::with_value(MessageCode::TLightFlasherStartTimedThenStayOn, 2.0),
            );
            simulation.queue_component_message(
                "bpr_solotgt_lights",
                TableMessage::from_code(MessageCode::TLightGroupResetAndTurnOn),
            );
            let group_mask = simulation.mark_current_player_right_hazard_target(mask);
            simulation.add_score(self.state.collision_score());
            if group_mask == 0b111 {
                simulation.queue_component_message(
                    "v_gate2",
                    TableMessage::from_code(MessageCode::TGateDisable),
                );
                simulation.queue_component_message(
                    "bpr_solotgt_lights",
                    TableMessage::with_value(MessageCode::TLightFlasherStartTimedThenStayOff, 2.0),
                );
                simulation.clear_current_player_right_hazard_targets();
            }
        } else if let Some(mask) = self.source_group_mask() {
            self.state.message_field |= mask;
            self.player_message_field_backup[self.current_player] = self.state.message_field;
            simulation.add_score(self.state.collision_score());
        } else {
            simulation.add_score(self.state.collision_score());
        }
        self.timer_remaining = Some(self.timer_time);
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
                self.timer_remaining = None;
                self.state.active = true;
                self.apply_active_visual();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::math::Vec2;

    use super::*;

    #[test]
    fn solo_target_disables_on_hard_hit_then_rearms() {
        let mut target = SoloTargetMechanic::new(ComponentId(1), "a_targ10");
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
        assert_eq!(target.state.sprite_index, 1);

        target.tick(&mut simulation, &table_state, 0.1);
        assert!(target.state.active);
        assert_eq!(target.state.sprite_index, 0);
        assert_eq!(simulation.score(), 0);
    }

    #[test]
    fn solo_target_ignores_soft_hit() {
        let mut target = SoloTargetMechanic::new(ComponentId(1), "a_targ10");
        let mut simulation = SimulationState::default();
        let table_state = TableInputState::default();

        target.on_collision(
            0,
            CollisionEdgeRole::Solid,
            CollisionContact {
                point: Vec2::ZERO,
                normal: Vec2::new(0.0, -1.0),
                distance: 0.0,
                impact_speed: 2.0,
                threshold_exceeded: false,
                owner_token: None,
                edge_role: CollisionEdgeRole::Solid,
            },
            &mut simulation,
            &table_state,
        );

        assert!(target.state.active);
        assert_eq!(target.state.sprite_index, 0);
    }

    #[test]
    fn solo_target_hard_hit_adds_score() {
        let mut target =
            SoloTargetMechanic::from_state(ComponentState::new(ComponentId(1), "a_targ10").with_scoring([750]));
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

        assert_eq!(simulation.score(), 750);
    }

    #[test]
    fn mission_spot_target_sets_source_shaped_bitmask_on_hit() {
        let mut target = SoloTargetMechanic::from_state(
            ComponentState::new(ComponentId(1), "a_targ14")
                .with_control("MissionSpotTargetControl")
                .with_scoring([1000]),
        );
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

        assert_eq!(target.state.message_field, 2);
    }

    #[test]
    fn mission_spot_target_preserves_message_field_per_player() {
        let mut target = SoloTargetMechanic::from_state(
            ComponentState::new(ComponentId(1), "a_targ13")
                .with_control("MissionSpotTargetControl")
                .with_scoring([1000]),
        );
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

        target.on_message(
            TableMessage::with_value(MessageCode::PlayerChanged, 1.0),
            &mut simulation,
            &table_state,
        );
        assert_eq!(target.state.message_field, 0);

        target.on_message(
            TableMessage::with_value(MessageCode::PlayerChanged, 0.0),
            &mut simulation,
            &table_state,
        );
        assert_eq!(target.state.message_field, 1);
    }

    #[test]
    fn left_hazard_spot_target_sets_source_shaped_bitmask_on_hit() {
        let mut target = SoloTargetMechanic::from_state(
            ComponentState::new(ComponentId(1), "a_targ17")
                .with_control("LeftHazardSpotTargetControl")
                .with_scoring([750]),
        );
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

        assert_eq!(target.state.message_field, 2);
    }

    #[test]
    fn right_hazard_spot_target_preserves_message_field_per_player() {
        let mut target = SoloTargetMechanic::from_state(
            ComponentState::new(ComponentId(1), "a_targ21")
                .with_control("RightHazardSpotTargetControl")
                .with_scoring([750]),
        );
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

        assert_eq!(target.state.message_field, 4);

        target.on_message(
            TableMessage::with_value(MessageCode::PlayerChanged, 1.0),
            &mut simulation,
            &table_state,
        );
        assert_eq!(target.state.message_field, 0);

        target.on_message(
            TableMessage::with_value(MessageCode::PlayerChanged, 0.0),
            &mut simulation,
            &table_state,
        );
        assert_eq!(target.state.message_field, 4);
    }

    #[test]
    fn fuel_spot_target_lights_assigned_lane_and_scores() {
        let mut target = SoloTargetMechanic::from_state(
            ComponentState::new(ComponentId(1), "a_targ10")
                .with_control("FuelSpotTargetControl")
                .with_scoring([750]),
        );
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

        assert_eq!(simulation.score(), 750);
        assert_eq!(simulation.current_player_fuel_spot_target_mask(), 0b001);
        let queued = simulation.drain_pending_component_messages();
        assert_eq!(queued.len(), 2);
        assert!(queued.iter().any(|(name, message)| {
            name == "lite70"
                && *message
                    == TableMessage::with_value(MessageCode::TLightFlasherStartTimedThenStayOn, 2.0)
        }));
        assert!(queued.iter().any(|(name, message)| {
            name == "top_circle_tgt_lights"
                && *message == TableMessage::from_code(MessageCode::TLightGroupResetAndTurnOn)
        }));
    }

    #[test]
    fn fuel_spot_target_completion_refuels_bargraph_and_clears_progress() {
        let mut target = SoloTargetMechanic::from_state(
            ComponentState::new(ComponentId(1), "a_targ12")
                .with_control("FuelSpotTargetControl")
                .with_scoring([750]),
        );
        let mut simulation = SimulationState::default();
        simulation.mark_current_player_fuel_spot_target(0b011);
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

        assert_eq!(simulation.score(), 750);
        assert_eq!(simulation.current_player_fuel_spot_target_mask(), 0);
        assert_eq!(simulation.info_text(), Some("Ship Re-Fueled"));

        let queued = simulation.drain_pending_component_messages();
        assert_eq!(queued.len(), 4);
        assert!(queued.iter().any(|(name, message)| {
            name == "fuel_bargraph"
                && *message == TableMessage::with_value(MessageCode::TLightGroupToggleSplitIndex, 11.0)
        }));
        assert!(queued.iter().any(|(name, message)| {
            name == "top_circle_tgt_lights"
                && *message == TableMessage::from_code(MessageCode::TLightGroupResetAndTurnOn)
        }));
        assert!(queued.iter().any(|(name, message)| {
            name == "top_circle_tgt_lights"
                && *message
                    == TableMessage::with_value(MessageCode::TLightFlasherStartTimedThenStayOff, 2.0)
        }));
    }

    #[test]
    fn mission_spot_target_drives_linked_light_and_group() {
        let mut target = SoloTargetMechanic::from_state(
            ComponentState::new(ComponentId(1), "a_targ14")
                .with_control("MissionSpotTargetControl")
                .with_scoring([1000]),
        );
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

        assert_eq!(target.state.message_field, 2);
        assert_eq!(simulation.score(), 1000);
        let queued = simulation.drain_pending_component_messages();
        assert_eq!(queued.len(), 2);
        assert!(queued.iter().any(|(name, message)| {
            name == "lite102"
                && *message
                    == TableMessage::with_value(MessageCode::TLightFlasherStartTimedThenStayOn, 2.0)
        }));
        assert!(queued.iter().any(|(name, message)| {
            name == "ramp_tgt_lights"
                && *message == TableMessage::from_code(MessageCode::TLightGroupResetAndTurnOn)
        }));
    }

    #[test]
    fn mission_spot_target_completion_flashes_group_off() {
        let mut target = SoloTargetMechanic::from_state(
            ComponentState::new(ComponentId(1), "a_targ15")
                .with_control("MissionSpotTargetControl")
                .with_scoring([1000]),
        );
        let mut simulation = SimulationState::default();
        let table_state = TableInputState::default();
        target.state.message_field = 0b011;
        target.player_message_field_backup[0] = 0b011;

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

        assert_eq!(target.state.message_field, 0b111);
        let queued = simulation.drain_pending_component_messages();
        assert!(queued.iter().any(|(name, message)| {
            name == "ramp_tgt_lights"
                && *message
                    == TableMessage::with_value(MessageCode::TLightFlasherStartTimedThenStayOff, 2.0)
        }));
    }

    #[test]
    fn left_hazard_target_drives_linked_light_and_group() {
        let mut target = SoloTargetMechanic::from_state(
            ComponentState::new(ComponentId(1), "a_targ17")
                .with_control("LeftHazardSpotTargetControl")
                .with_scoring([750]),
        );
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

        assert_eq!(target.state.message_field, 2);
        assert_eq!(simulation.score(), 750);
        assert_eq!(simulation.current_player_left_hazard_target_mask(), 0b010);
        let queued = simulation.drain_pending_component_messages();
        assert_eq!(queued.len(), 2);
        assert!(queued.iter().any(|(name, message)| {
            name == "lite105"
                && *message
                    == TableMessage::with_value(MessageCode::TLightFlasherStartTimedThenStayOn, 2.0)
        }));
        assert!(queued.iter().any(|(name, message)| {
            name == "lchute_tgt_lights"
                && *message == TableMessage::from_code(MessageCode::TLightGroupResetAndTurnOn)
        }));
    }

    #[test]
    fn left_hazard_target_completion_disables_left_gate() {
        let mut target = SoloTargetMechanic::from_state(
            ComponentState::new(ComponentId(1), "a_targ18")
                .with_control("LeftHazardSpotTargetControl")
                .with_scoring([750]),
        );
        let mut simulation = SimulationState::default();
        simulation.mark_current_player_left_hazard_target(0b011);
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

        assert_eq!(simulation.current_player_left_hazard_target_mask(), 0);
        let queued = simulation.drain_pending_component_messages();
        assert!(queued.iter().any(|(name, message)| {
            name == "v_gate1" && *message == TableMessage::from_code(MessageCode::TGateDisable)
        }));
        assert!(queued.iter().any(|(name, message)| {
            name == "lchute_tgt_lights"
                && *message
                    == TableMessage::with_value(MessageCode::TLightFlasherStartTimedThenStayOff, 2.0)
        }));
    }

    #[test]
    fn right_hazard_target_completion_disables_right_gate() {
        let mut target = SoloTargetMechanic::from_state(
            ComponentState::new(ComponentId(1), "a_targ21")
                .with_control("RightHazardSpotTargetControl")
                .with_scoring([750]),
        );
        let mut simulation = SimulationState::default();
        simulation.mark_current_player_right_hazard_target(0b011);
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

        assert_eq!(simulation.current_player_right_hazard_target_mask(), 0);
        let queued = simulation.drain_pending_component_messages();
        assert!(queued.iter().any(|(name, message)| {
            name == "v_gate2" && *message == TableMessage::from_code(MessageCode::TGateDisable)
        }));
        assert!(queued.iter().any(|(name, message)| {
            name == "bpr_solotgt_lights"
                && *message
                    == TableMessage::with_value(MessageCode::TLightFlasherStartTimedThenStayOff, 2.0)
        }));
    }
}
