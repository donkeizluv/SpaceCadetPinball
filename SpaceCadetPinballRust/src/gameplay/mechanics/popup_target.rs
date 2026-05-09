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
    const BOOSTER_TARGET_NAMES: [&'static str; 3] = ["a_targ1", "a_targ2", "a_targ3"];
    const MEDAL_TARGET_NAMES: [&'static str; 3] = ["a_targ4", "a_targ5", "a_targ6"];
    const MULTIPLIER_TARGET_NAMES: [&'static str; 3] = ["a_targ7", "a_targ8", "a_targ9"];

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

    fn mark_hit_for_current_player(&mut self) {
        self.state.message_field = 1;
        self.player_message_field_backup[self.current_player] = self.state.message_field;
    }

    fn score_at(&self, index: usize) -> u64 {
        self.state
            .scoring
            .get(index)
            .copied()
            .unwrap_or(0)
            .max(0) as u64
    }

    fn is_multiplier_target_control(&self) -> bool {
        self.state.control_name == Some("MultiplierTargetControl")
    }

    fn is_medal_target_control(&self) -> bool {
        self.state.control_name == Some("MedalTargetControl")
    }

    fn is_booster_target_control(&self) -> bool {
        self.state.control_name == Some("BoosterTargetControl")
    }

    fn booster_target_mask(&self) -> Option<u8> {
        if !self.is_booster_target_control() {
            return None;
        }

        match self.state.group_name.as_str() {
            "a_targ1" => Some(0b001),
            "a_targ2" => Some(0b010),
            "a_targ3" => Some(0b100),
            _ => None,
        }
    }

    fn medal_target_mask(&self) -> Option<u8> {
        if !self.is_medal_target_control() {
            return None;
        }

        match self.state.group_name.as_str() {
            "a_targ4" => Some(0b001),
            "a_targ5" => Some(0b010),
            "a_targ6" => Some(0b100),
            _ => None,
        }
    }

    fn multiplier_target_mask(&self) -> Option<u8> {
        if !self.is_multiplier_target_control() {
            return None;
        }

        match self.state.group_name.as_str() {
            "a_targ7" => Some(0b001),
            "a_targ8" => Some(0b010),
            "a_targ9" => Some(0b100),
            _ => None,
        }
    }

    fn multiplier_text(multiplier_index: u8) -> &'static str {
        match multiplier_index {
            1 => "2X MULTIPLIER",
            2 => "3X MULTIPLIER",
            3 => "5X MULTIPLIER",
            _ => "10X MULTIPLIER",
        }
    }

    fn medal_text(medal_level: u8) -> &'static str {
        match medal_level {
            0 => "Level One Commendation",
            1 => "Level Two Commendation",
            _ => "Extra Ball",
        }
    }

    fn booster_text(booster_level: u8) -> &'static str {
        match booster_level {
            0 => "Flags Upgraded",
            1 => "Jackpot Activated",
            2 => "Bonus Activated",
            3 => "Bonus Hold",
            _ => "Booster Maxed",
        }
    }

    fn queue_light_message(
        simulation: &mut SimulationState,
        light_name: &'static str,
        message: TableMessage,
    ) {
        simulation.queue_component_message(light_name, message);
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
                self.state.message_field = 0;
                self.player_message_field_backup[self.current_player] = 0;
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
        self.mark_hit_for_current_player();
        if let Some(mask) = self.booster_target_mask() {
            let group_mask = simulation.mark_current_player_booster_target(mask);
            if group_mask == 0b111 {
                let booster_level = simulation.current_player_booster_level();
                simulation.add_score(self.score_at(1));
                match booster_level {
                    0 => {
                        Self::queue_light_message(
                            simulation,
                            "lite61",
                            TableMessage::with_value(MessageCode::TLightTurnOnTimed, 60.0),
                        );
                        simulation.display_info_text(Self::booster_text(booster_level), 2.0);
                    }
                    1 => {
                        simulation.activate_jackpot_score();
                        Self::queue_light_message(
                            simulation,
                            "lite60",
                            TableMessage::with_value(MessageCode::TLightTurnOnTimed, 60.0),
                        );
                        simulation.display_info_text(Self::booster_text(booster_level), 2.0);
                    }
                    2 => {
                        simulation.activate_bonus_score();
                        Self::queue_light_message(
                            simulation,
                            "lite59",
                            TableMessage::with_value(MessageCode::TLightTurnOnTimed, 60.0),
                        );
                        simulation.display_info_text(Self::booster_text(booster_level), 2.0);
                    }
                    3 => {
                        simulation.activate_bonus_hold();
                        Self::queue_light_message(
                            simulation,
                            "lite58",
                            TableMessage::from_code(MessageCode::TLightResetAndTurnOn),
                        );
                        simulation.display_info_text(Self::booster_text(booster_level), 2.0);
                    }
                    _ => {
                        simulation.add_score(self.score_at(1));
                        simulation.display_info_text(Self::booster_text(booster_level), 2.0);
                    }
                }
                simulation.advance_current_player_booster_level();
                simulation.clear_current_player_booster_targets();
                for target_name in Self::BOOSTER_TARGET_NAMES {
                    simulation.queue_component_message(
                        target_name,
                        TableMessage::from_code(MessageCode::TPopupTargetEnable),
                    );
                }
            } else {
                simulation.add_score(self.score_at(0));
            }
        } else if let Some(mask) = self.medal_target_mask() {
            let group_mask = simulation.mark_current_player_medal_target(mask);
            if group_mask == 0b111 {
                let medal_level = simulation.current_player_medal_level();
                match medal_level {
                    0 => simulation.add_score(self.score_at(1)),
                    1 => simulation.add_score(self.score_at(2)),
                    _ => simulation.add_extra_ball(),
                }
                simulation.display_info_text(Self::medal_text(medal_level), 2.0);
                simulation.advance_current_player_medal_level();
                simulation.clear_current_player_medal_targets();
                simulation.queue_component_message(
                    "bumper_target_lights",
                    TableMessage::from_code(MessageCode::TLightGroupResetAndTurnOn),
                );
                simulation.queue_component_message(
                    "bumper_target_lights",
                    TableMessage::with_value(MessageCode::TLightGroupRestartNotifyTimer, 30.0),
                );
                for target_name in Self::MEDAL_TARGET_NAMES {
                    simulation.queue_component_message(
                        target_name,
                        TableMessage::from_code(MessageCode::TPopupTargetEnable),
                    );
                }
            } else {
                simulation.add_score(self.score_at(0));
            }
        } else if let Some(mask) = self.multiplier_target_mask() {
            let group_mask = simulation.mark_current_player_multiplier_target(mask);
            if group_mask == 0b111 {
                simulation.add_score(self.score_at(1));
                simulation.clear_current_player_multiplier_targets();
                simulation.score_multiplier = simulation.score_multiplier.saturating_add(1).clamp(1, 4);
                simulation.display_info_text(
                    Self::multiplier_text(simulation.score_multiplier),
                    2.0,
                );
                simulation.queue_component_message(
                    "top_target_lights",
                    TableMessage::from_code(MessageCode::TLightGroupResetAndTurnOn),
                );
                simulation.queue_component_message(
                    "top_target_lights",
                    TableMessage::with_value(MessageCode::TLightGroupRestartNotifyTimer, 30.0),
                );
                for target_name in Self::MULTIPLIER_TARGET_NAMES {
                    simulation.queue_component_message(
                        target_name,
                        TableMessage::from_code(MessageCode::TPopupTargetEnable),
                    );
                }
            } else {
                simulation.add_score(self.score_at(0));
            }
        } else {
            simulation.add_score(self.state.collision_score());
        }
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

    #[test]
    fn multiplier_target_trio_completion_awards_group_score_and_advances_multiplier() {
        let table_state = TableInputState::default();
        let mut simulation = SimulationState::default();
        let mut target7 = PopupTargetMechanic::from_state(
            ComponentState::new(ComponentId(1), "a_targ7")
                .with_control("MultiplierTargetControl")
                .with_scoring([500, 1500]),
        );
        let mut target8 = PopupTargetMechanic::from_state(
            ComponentState::new(ComponentId(2), "a_targ8")
                .with_control("MultiplierTargetControl")
                .with_scoring([500, 1500]),
        );
        let mut target9 = PopupTargetMechanic::from_state(
            ComponentState::new(ComponentId(3), "a_targ9")
                .with_control("MultiplierTargetControl")
                .with_scoring([500, 1500]),
        );
        let contact = CollisionContact {
            point: Vec2::ZERO,
            normal: Vec2::new(0.0, -1.0),
            distance: 0.0,
            impact_speed: 12.0,
            threshold_exceeded: true,
            owner_token: None,
            edge_role: CollisionEdgeRole::Solid,
        };

        target7.on_collision(0, CollisionEdgeRole::Solid, contact, &mut simulation, &table_state);
        target8.on_collision(0, CollisionEdgeRole::Solid, contact, &mut simulation, &table_state);
        target9.on_collision(0, CollisionEdgeRole::Solid, contact, &mut simulation, &table_state);

        assert_eq!(simulation.score(), 2_500);
        assert_eq!(simulation.score_multiplier, 1);
        assert_eq!(simulation.current_player_multiplier_target_mask(), 0);
        assert_eq!(simulation.info_text(), Some("2X MULTIPLIER"));

        let queued = simulation.drain_pending_component_messages();
        assert_eq!(queued.len(), 5);
        assert!(queued.iter().filter(|(name, _)| name.starts_with("a_targ")).count() == 3);
        assert!(queued.iter().any(|(name, message)| {
            name == "top_target_lights"
                && matches!(message, TableMessage::Code(MessageCode::TLightGroupResetAndTurnOn, _))
        }));
    }

    #[test]
    fn popup_target_enable_clears_current_player_latched_state() {
        let mut target = PopupTargetMechanic::new(ComponentId(1), "a_targ7");
        let mut simulation = SimulationState::default();
        let table_state = TableInputState::default();

        target.state.message_field = 1;
        target.player_message_field_backup[0] = 1;
        target.disable();
        target.on_message(
            TableMessage::from_code(MessageCode::TPopupTargetEnable),
            &mut simulation,
            &table_state,
        );

        assert_eq!(target.state.message_field, 0);
        target.tick(&mut simulation, &table_state, 0.1);
        assert!(target.state.active);
    }

    #[test]
    fn medal_target_trio_first_completion_awards_level_one_score() {
        let table_state = TableInputState::default();
        let mut simulation = SimulationState::default();
        let mut target4 = PopupTargetMechanic::from_state(
            ComponentState::new(ComponentId(1), "a_targ4")
                .with_control("MedalTargetControl")
                .with_scoring([1500, 10_000, 50_000]),
        );
        let mut target5 = PopupTargetMechanic::from_state(
            ComponentState::new(ComponentId(2), "a_targ5")
                .with_control("MedalTargetControl")
                .with_scoring([1500, 10_000, 50_000]),
        );
        let mut target6 = PopupTargetMechanic::from_state(
            ComponentState::new(ComponentId(3), "a_targ6")
                .with_control("MedalTargetControl")
                .with_scoring([1500, 10_000, 50_000]),
        );
        let contact = CollisionContact {
            point: Vec2::ZERO,
            normal: Vec2::new(0.0, -1.0),
            distance: 0.0,
            impact_speed: 12.0,
            threshold_exceeded: true,
            owner_token: None,
            edge_role: CollisionEdgeRole::Solid,
        };

        target4.on_collision(0, CollisionEdgeRole::Solid, contact, &mut simulation, &table_state);
        target5.on_collision(0, CollisionEdgeRole::Solid, contact, &mut simulation, &table_state);
        target6.on_collision(0, CollisionEdgeRole::Solid, contact, &mut simulation, &table_state);

        assert_eq!(simulation.score(), 13_000);
        assert_eq!(simulation.current_player_medal_target_mask(), 0);
        assert_eq!(simulation.current_player_medal_level(), 1);
        assert_eq!(simulation.info_text(), Some("Level One Commendation"));
        assert_eq!(simulation.player_scores[0].extra_balls, 0);

        let queued = simulation.drain_pending_component_messages();
        assert_eq!(queued.len(), 5);
        assert!(queued.iter().any(|(name, message)| {
            name == "bumper_target_lights"
                && matches!(message, TableMessage::Code(MessageCode::TLightGroupResetAndTurnOn, _))
        }));
    }

    #[test]
    fn medal_target_trio_third_completion_awards_extra_ball() {
        let table_state = TableInputState::default();
        let mut simulation = SimulationState::default();
        simulation.player_scores[0].medal_level = 2;
        let mut target4 = PopupTargetMechanic::from_state(
            ComponentState::new(ComponentId(1), "a_targ4")
                .with_control("MedalTargetControl")
                .with_scoring([1500, 10_000, 50_000]),
        );
        let mut target5 = PopupTargetMechanic::from_state(
            ComponentState::new(ComponentId(2), "a_targ5")
                .with_control("MedalTargetControl")
                .with_scoring([1500, 10_000, 50_000]),
        );
        let mut target6 = PopupTargetMechanic::from_state(
            ComponentState::new(ComponentId(3), "a_targ6")
                .with_control("MedalTargetControl")
                .with_scoring([1500, 10_000, 50_000]),
        );
        let contact = CollisionContact {
            point: Vec2::ZERO,
            normal: Vec2::new(0.0, -1.0),
            distance: 0.0,
            impact_speed: 12.0,
            threshold_exceeded: true,
            owner_token: None,
            edge_role: CollisionEdgeRole::Solid,
        };

        target4.on_collision(0, CollisionEdgeRole::Solid, contact, &mut simulation, &table_state);
        target5.on_collision(0, CollisionEdgeRole::Solid, contact, &mut simulation, &table_state);
        target6.on_collision(0, CollisionEdgeRole::Solid, contact, &mut simulation, &table_state);

        assert_eq!(simulation.score(), 3_000);
        assert_eq!(simulation.player_scores[0].extra_balls, 1);
        assert_eq!(simulation.current_player_medal_target_mask(), 0);
        assert_eq!(simulation.current_player_medal_level(), 2);
        assert_eq!(simulation.info_text(), Some("Extra Ball"));
    }

    #[test]
    fn booster_target_trio_second_completion_activates_jackpot() {
        let table_state = TableInputState::default();
        let mut simulation = SimulationState::default();
        simulation.player_scores[0].booster_level = 1;
        let mut target1 = PopupTargetMechanic::from_state(
            ComponentState::new(ComponentId(1), "a_targ1")
                .with_control("BoosterTargetControl")
                .with_scoring([500, 5000]),
        );
        let mut target2 = PopupTargetMechanic::from_state(
            ComponentState::new(ComponentId(2), "a_targ2")
                .with_control("BoosterTargetControl")
                .with_scoring([500, 5000]),
        );
        let mut target3 = PopupTargetMechanic::from_state(
            ComponentState::new(ComponentId(3), "a_targ3")
                .with_control("BoosterTargetControl")
                .with_scoring([500, 5000]),
        );
        let contact = CollisionContact {
            point: Vec2::ZERO,
            normal: Vec2::new(0.0, -1.0),
            distance: 0.0,
            impact_speed: 12.0,
            threshold_exceeded: true,
            owner_token: None,
            edge_role: CollisionEdgeRole::Solid,
        };

        target1.on_collision(0, CollisionEdgeRole::Solid, contact, &mut simulation, &table_state);
        target2.on_collision(0, CollisionEdgeRole::Solid, contact, &mut simulation, &table_state);
        target3.on_collision(0, CollisionEdgeRole::Solid, contact, &mut simulation, &table_state);

        assert_eq!(simulation.score(), 6_000);
        assert!(simulation.jackpot_score_active);
        assert_eq!(simulation.current_player_booster_target_mask(), 0);
        assert_eq!(simulation.current_player_booster_level(), 2);
        assert_eq!(simulation.info_text(), Some("Jackpot Activated"));
        let queued = simulation.drain_pending_component_messages();
        assert!(queued.iter().any(|(name, message)| {
            name == "lite60"
                && matches!(message, TableMessage::Code(MessageCode::TLightTurnOnTimed, value) if *value == 60.0)
        }));
    }

    #[test]
    fn booster_target_trio_fifth_completion_awards_double_top_score() {
        let table_state = TableInputState::default();
        let mut simulation = SimulationState::default();
        simulation.player_scores[0].booster_level = 4;
        let mut target1 = PopupTargetMechanic::from_state(
            ComponentState::new(ComponentId(1), "a_targ1")
                .with_control("BoosterTargetControl")
                .with_scoring([500, 5000]),
        );
        let mut target2 = PopupTargetMechanic::from_state(
            ComponentState::new(ComponentId(2), "a_targ2")
                .with_control("BoosterTargetControl")
                .with_scoring([500, 5000]),
        );
        let mut target3 = PopupTargetMechanic::from_state(
            ComponentState::new(ComponentId(3), "a_targ3")
                .with_control("BoosterTargetControl")
                .with_scoring([500, 5000]),
        );
        let contact = CollisionContact {
            point: Vec2::ZERO,
            normal: Vec2::new(0.0, -1.0),
            distance: 0.0,
            impact_speed: 12.0,
            threshold_exceeded: true,
            owner_token: None,
            edge_role: CollisionEdgeRole::Solid,
        };

        target1.on_collision(0, CollisionEdgeRole::Solid, contact, &mut simulation, &table_state);
        target2.on_collision(0, CollisionEdgeRole::Solid, contact, &mut simulation, &table_state);
        target3.on_collision(0, CollisionEdgeRole::Solid, contact, &mut simulation, &table_state);

        assert_eq!(simulation.score(), 11_000);
        assert_eq!(simulation.current_player_booster_target_mask(), 0);
        assert_eq!(simulation.current_player_booster_level(), 4);
        assert_eq!(simulation.info_text(), Some("Booster Maxed"));
    }
}
