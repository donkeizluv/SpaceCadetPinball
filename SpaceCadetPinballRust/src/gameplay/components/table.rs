mod builder;
mod input;
mod text_box;
mod visuals;

use std::collections::HashMap;

use crate::assets::{DatFile, VisualCollisionEdge};
use crate::engine::TableBridgeState;
use crate::engine::math::Vec2;
use crate::engine::physics::{
    Ball, CollisionComponentMetadata, CollisionComponentRegistry, CollisionContact,
    CollisionResponseParams, EdgeManager,
};

use super::component::{CollisionGeometryKind, GameplayComponent};
use super::group::{ComponentGroup, ComponentId};
use super::messages::{MessageCode, TableMessage};

pub use builder::{
    ComponentDefinition, ComponentKind, TableLinkReport, default_component_definitions,
};
pub use input::TableInputState;
pub use visuals::{
    BitmapVisualState, HudVisualState, LightVisualState, NumberWidgetVisualState,
    SequenceVisualState, TableVisual, TableVisualState, TextBoxVisualState,
};

use text_box::TextBoxState;

pub struct PinballTable {
    components: ComponentGroup,
    component_slots: HashMap<ComponentId, Box<dyn GameplayComponent>>,
    input_state: TableInputState,
    message_log: Vec<TableMessage>,
    simulation: SimulationState,
    link_report: TableLinkReport,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TableRegionState {
    pub lane_ready: f32,
    pub ball_x: f32,
    pub ball_y: f32,
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
    pub ramp: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TableVisualSignalState {
    pub score_progress: f32,
    pub launch_progress: f32,
    pub drain_progress: f32,
    pub impact_focus: f32,
    pub recovery_focus: f32,
    pub lane_focus: f32,
    pub hazard_focus: f32,
    pub target_focus: f32,
    pub orbit_focus: f32,
    pub field_light_focus: f32,
    pub rollover_light_focus: f32,
    pub fuel_focus: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TableActivityState {
    pub ramp_activity: f32,
    pub lower_hazard_activity: f32,
    pub orbit_activity: f32,
    pub target_activity: f32,
    pub bumper_activity: f32,
    pub lane_activity: f32,
}

pub struct SimulationState {
    pub balls: Vec<Ball>,
    pub player_scores: [PlayerState; PlayerState::MAX_PLAYERS],
    pub player_count: u8,
    pub current_player: u8,
    pub max_ball_count: u8,
    pub score_multiplier: u8,
    pub score_added: u64,
    pub bonus_score: u64,
    pub bonus_score_active: bool,
    pub jackpot_score_active: bool,
    pub game_over: bool,
    pub collision_component_offset: f32,
    pub plunger_charge: f32,
    pub launch_count: u64,
    pub drain_count: u64,
    pub ball_in_drain: bool,
    pub multiball_count: u32,
    pub tilt_locked: bool,
    pub left_flipper_active: bool,
    pub right_flipper_active: bool,
    pub regions: TableRegionState,
    pub visual_signals: TableVisualSignalState,
    pub activities: TableActivityState,
    pub edge_manager: EdgeManager,
    pub collision_components: CollisionComponentRegistry,
    pub plunger_position: Vec2,
    info_box: TextBoxState,
    mission_box: TextBoxState,
    pending_messages: Vec<TableMessage>,
    game_over_timer_remaining: Option<f32>,
    game_over_ready_for_restart: bool,
    previous_ball_present: bool,
    previous_launch_count: u64,
    previous_drain_count: u64,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PlayerState {
    pub score: u64,
    pub score_e9_part: u32,
    pub ball_count: u8,
    pub extra_balls: u8,
    pub jackpot_score: u64,
}

impl PlayerState {
    const MAX_PLAYERS: usize = 4;

    const fn new(ball_count: u8) -> Self {
        Self {
            score: 0,
            score_e9_part: 0,
            ball_count,
            extra_balls: 0,
            jackpot_score: 0,
        }
    }
}

impl Default for SimulationState {
    fn default() -> Self {
        let max_ball_count = 3;
        Self {
            balls: Vec::new(),
            player_scores: [PlayerState::new(max_ball_count); PlayerState::MAX_PLAYERS],
            player_count: 1,
            current_player: 0,
            max_ball_count,
            score_multiplier: 0,
            score_added: 0,
            bonus_score: 10_000,
            bonus_score_active: false,
            jackpot_score_active: false,
            game_over: false,
            collision_component_offset: 6.0,
            plunger_charge: 0.0,
            launch_count: 0,
            drain_count: 0,
            ball_in_drain: false,
            multiball_count: 0,
            tilt_locked: false,
            left_flipper_active: false,
            right_flipper_active: false,
            regions: TableRegionState::default(),
            visual_signals: TableVisualSignalState::default(),
            activities: TableActivityState::default(),
            edge_manager: EdgeManager::for_table_bounds(600.0, 416.0),
            collision_components: CollisionComponentRegistry::default(),
            plunger_position: Vec2::new(560.0, 382.0),
            info_box: TextBoxState::default(),
            mission_box: TextBoxState::default(),
            pending_messages: Vec::new(),
            game_over_timer_remaining: None,
            game_over_ready_for_restart: false,
            previous_ball_present: false,
            previous_launch_count: 0,
            previous_drain_count: 0,
        }
    }
}

impl Default for PinballTable {
    fn default() -> Self {
        Self {
            components: ComponentGroup::default(),
            component_slots: HashMap::new(),
            input_state: TableInputState::default(),
            message_log: Vec::new(),
            simulation: SimulationState::default(),
            link_report: TableLinkReport::default(),
        }
    }
}

impl PinballTable {
    fn edge_owner_token(component_id: ComponentId, slot: u8) -> u32 {
        ((component_id.0 as u32) << 8) | u32::from(slot)
    }

    fn decode_edge_owner_token(token: u32) -> (ComponentId, u8) {
        (ComponentId((token >> 8) as usize), (token & 0xFF) as u8)
    }

    pub fn new() -> Self {
        Self::from_component_definitions(&default_component_definitions(), None)
    }

    pub fn from_dat(dat_file: &DatFile) -> Self {
        Self::from_component_definitions(&default_component_definitions(), Some(dat_file))
    }

    pub fn from_component_definitions(
        definitions: &[ComponentDefinition],
        dat_file: Option<&DatFile>,
    ) -> Self {
        let mut table = Self::default();
        table.link_report = builder::install_components(&mut table, definitions, dat_file);
        if let Some(dat_file) = dat_file {
            table.resolve_plunger_position(dat_file);
            table.apply_component_float_attributes(dat_file);
            table.register_collision_metadata(dat_file);
            table.register_collision_edges(dat_file);
        }
        table.simulation.refresh_derived_state();
        table.refresh_text_boxes();
        table
    }

    pub fn components(&self) -> &ComponentGroup {
        &self.components
    }

    pub fn register_component(&mut self, id: ComponentId, name: impl Into<String>) {
        self.components.register(id, name);
    }

    pub fn register_component_with_group_index(
        &mut self,
        id: ComponentId,
        name: impl Into<String>,
        group_index: Option<i32>,
    ) {
        self.components
            .register_with_group_index(id, name, group_index);
    }

    pub fn add_component(&mut self, component: impl GameplayComponent + 'static) {
        self.add_boxed_component(Box::new(component));
    }

    pub fn add_boxed_component(&mut self, component: Box<dyn GameplayComponent>) {
        let id = component.id();
        let name = component.name().to_string();
        let group_index = component.group_index();
        self.components
            .register_with_group_index(id, name, group_index);
        self.component_slots.insert(id, component);
    }

    pub fn component(&self, id: ComponentId) -> Option<&dyn GameplayComponent> {
        self.component_slots.get(&id).map(Box::as_ref)
    }

    pub fn component_mut(&mut self, id: ComponentId) -> Option<&mut (dyn GameplayComponent + '_)> {
        match self.component_slots.get_mut(&id) {
            Some(component) => Some(component.as_mut()),
            None => None,
        }
    }

    pub fn find_component(&self, name: &str) -> Option<&dyn GameplayComponent> {
        self.components.find(name).and_then(|id| self.component(id))
    }

    pub fn find_component_by_group_index(
        &self,
        group_index: i32,
    ) -> Option<&dyn GameplayComponent> {
        self.components
            .find_by_group_index(group_index)
            .and_then(|id| self.component(id))
    }

    pub fn component_count(&self) -> usize {
        self.component_slots.len()
    }

    pub fn collision_component_count(&self) -> usize {
        self.simulation.collision_components.len()
    }

    pub fn collision_wall_count(&self) -> usize {
        self.simulation.edge_manager.wall_count()
    }

    pub fn link_report(&self) -> &TableLinkReport {
        &self.link_report
    }

    pub fn active_ball(&self) -> Option<&Ball> {
        self.simulation.active_ball()
    }

    pub fn ball_count_in_rect(&self, min: Vec2, max: Vec2) -> usize {
        self.simulation.ball_count_in_rect(min, max)
    }

    pub fn launch_count(&self) -> u64 {
        self.simulation.launch_count
    }

    pub fn drain_count(&self) -> u64 {
        self.simulation.drain_count
    }

    pub fn score(&self) -> u64 {
        self.simulation.score()
    }

    pub fn input_state(&self) -> TableInputState {
        self.input_state
    }

    pub fn message_log(&self) -> &[TableMessage] {
        &self.message_log
    }

    pub fn clear_message_log(&mut self) {
        self.message_log.clear();
    }

    pub fn dispatch(&mut self, message: TableMessage) {
        match message {
            TableMessage::LeftFlipperPressed => self.input_state.left_flipper = true,
            TableMessage::LeftFlipperReleased => self.input_state.left_flipper = false,
            TableMessage::RightFlipperPressed => self.input_state.right_flipper = true,
            TableMessage::RightFlipperReleased => self.input_state.right_flipper = false,
            TableMessage::PlungerPressed => self.input_state.plunger_pulling = true,
            TableMessage::PlungerReleased => self.input_state.plunger_pulling = false,
            TableMessage::StartGame => self.input_state.pending_start = true,
            TableMessage::Nudge(vector) => self.input_state.pending_nudge = Some(vector),
            TableMessage::Pause | TableMessage::Resume => {}
            TableMessage::Code(code, _) => match code {
                MessageCode::LeftFlipperInputPressed => self.input_state.left_flipper = true,
                MessageCode::LeftFlipperInputReleased => self.input_state.left_flipper = false,
                MessageCode::RightFlipperInputPressed => self.input_state.right_flipper = true,
                MessageCode::RightFlipperInputReleased => self.input_state.right_flipper = false,
                MessageCode::PlungerInputPressed => self.input_state.plunger_pulling = true,
                MessageCode::PlungerInputReleased => self.input_state.plunger_pulling = false,
                MessageCode::StartGamePlayer1 | MessageCode::NewGame => {
                    self.input_state.pending_start = true;
                }
                MessageCode::Pause | MessageCode::Resume => {}
                _ => {}
            },
        }

        let deferred_player_changed = self.handle_table_message(message);
        self.message_log.push(message);
        self.broadcast_message(message);
        if let Some(player_changed) = deferred_player_changed {
            self.message_log.push(player_changed);
            self.broadcast_message(player_changed);
        }
        self.simulation.refresh_derived_state();
    }

    pub fn sync_bridge_state(&mut self, bridge_state: &TableBridgeState) {
        let previous = TableBridgeState {
            left_flipper: self.input_state.left_flipper,
            right_flipper: self.input_state.right_flipper,
            plunger_pulling: self.input_state.plunger_pulling,
            last_release_impulse: 0.0,
            pending_start: false,
            pending_nudge: None,
            input_ticks: self.input_state.ticks,
        };

        for message in TableMessage::from_bridge_state(bridge_state, &previous) {
            self.dispatch(message);
        }

        self.input_state = TableInputState::from(bridge_state);
    }

    pub fn tick_components(&mut self, dt: f32) {
        let table_state = self.input_state;
        for component_id in self.components.iter() {
            if let Some(component) = self.component_slots.get_mut(&component_id) {
                component.tick(&mut self.simulation, &table_state, dt);
            }
        }
        self.process_pending_simulation_messages();
        self.simulation.tick_table_timers(dt);
        self.simulation.refresh_derived_state();
    }

    pub fn step_simulation(&mut self, dt: f32) {
        self.simulation.edge_manager.set_flipper_state(
            self.simulation.left_flipper_active,
            self.simulation.right_flipper_active,
        );

        let pending_nudge = self.input_state.pending_nudge.take();
        for ball_index in 0..self.simulation.balls.len() {
            let collision_events = {
                let component_slots = &self.component_slots;
                let edge_manager = &self.simulation.edge_manager;
                let ball = &mut self.simulation.balls[ball_index];
                if let Some(nudge) = pending_nudge {
                    ball.apply_nudge(nudge);
                }
                ball.step(dt);
                edge_manager.prepare_collision_pass(ball);

                let mut events = Vec::new();
                if let Some(contact) = edge_manager.resolve_ball_with_context(
                    ball,
                    |owner_token| match owner_token {
                        Some(owner_token) => {
                            let (component_id, slot) = Self::decode_edge_owner_token(owner_token);
                            component_slots
                                .get(&component_id)
                                .is_none_or(|component| component.collision_edge_active(slot))
                        }
                        None => true,
                    },
                    |owner_token| match owner_token {
                        Some(owner_token) => {
                            let (component_id, _) = Self::decode_edge_owner_token(owner_token);
                            self.simulation
                                .collision_components
                                .get(component_id)
                                .map(|metadata| CollisionResponseParams {
                                    elasticity: metadata.elasticity,
                                    smoothness: metadata.smoothness,
                                    threshold: metadata.threshold,
                                    boost: metadata.boost,
                                })
                                .unwrap_or_default()
                        }
                        None => CollisionResponseParams {
                            elasticity: 0.82,
                            ..CollisionResponseParams::default()
                        },
                    },
                )
                {
                    events.push(contact);
                }
                events.extend(edge_manager.trigger_contacts_with_filter(
                    ball,
                    |owner_token| match owner_token {
                        Some(owner_token) => {
                            let (component_id, slot) = Self::decode_edge_owner_token(owner_token);
                            component_slots
                                .get(&component_id)
                                .is_none_or(|component| component.collision_edge_active(slot))
                        }
                        None => true,
                    },
                ));
                events
            };

            for contact in collision_events {
                self.dispatch_collision_contact(contact);
            }
        }

        self.process_pending_simulation_messages();
        self.input_state.pending_start = false;
        self.simulation.tick_table_timers(dt);
        self.simulation.update_activity_state(dt);
        self.simulation.refresh_derived_state();
        self.simulation.info_box.tick(dt);
        self.simulation.mission_box.tick(dt);
        self.refresh_text_boxes();
    }

    fn broadcast_message(&mut self, message: TableMessage) {
        let table_state = self.input_state;
        for component_id in self.components.iter() {
            if let Some(component) = self.component_slots.get_mut(&component_id) {
                component.on_message(message, &mut self.simulation, &table_state);
            }
        }
    }

    fn process_pending_simulation_messages(&mut self) {
        for message in self.simulation.drain_pending_messages() {
            self.dispatch(message);
        }
    }

    fn handle_table_message(&mut self, message: TableMessage) -> Option<TableMessage> {
        match message {
            TableMessage::Code(MessageCode::StartGamePlayer1, value)
            | TableMessage::Code(MessageCode::NewGame, value) => {
                let requested_players = value.floor().clamp(1.0, PlayerState::MAX_PLAYERS as f32) as u8;
                self.simulation.start_new_game(requested_players);
                None
            }
            TableMessage::Code(MessageCode::SwitchToNextPlayer, _) => self
                .simulation
                .switch_to_next_player()
                .map(|next_player| {
                    TableMessage::with_value(MessageCode::PlayerChanged, f32::from(next_player))
                }),
            TableMessage::Code(MessageCode::PlayerChanged, value) => {
                let next_player =
                    value.floor().clamp(0.0, (PlayerState::MAX_PLAYERS - 1) as f32) as u8;
                self.simulation.set_current_player(next_player);
                None
            }
            TableMessage::Code(MessageCode::Reset, _) => {
                self.simulation.reset_player_state();
                None
            }
            TableMessage::Code(MessageCode::GameOver, _) => {
                self.simulation.enter_game_over();
                None
            }
            _ => None,
        }
    }

    fn dispatch_collision_contact(&mut self, contact: CollisionContact) {
        let Some(owner_token) = contact.owner_token else {
            return;
        };
        let (component_id, slot) = Self::decode_edge_owner_token(owner_token);
        let table_state = self.input_state;
        if let Some(component) = self.component_slots.get_mut(&component_id) {
            component.on_collision(
                slot,
                contact.edge_role,
                contact,
                &mut self.simulation,
                &table_state,
            );
        }
    }

    fn register_collision_metadata(&mut self, dat_file: &DatFile) {
        for component in self.component_slots.values() {
            let Some(group_index) = component.group_index() else {
                continue;
            };
            if group_index < 0 {
                continue;
            }

            let Some(metadata) = dat_file.visual_collision_metadata(group_index as usize, 0) else {
                continue;
            };

            self.simulation
                .collision_components
                .register(CollisionComponentMetadata {
                    component_id: component.id(),
                    group_index,
                    collision_group: metadata.collision_group,
                    smoothness: metadata.smoothness,
                    elasticity: metadata.elasticity,
                    threshold: metadata.threshold,
                    boost: metadata.boost,
                    soft_hit_sound_id: metadata.soft_hit_sound_id,
                    hard_hit_sound_id: metadata.hard_hit_sound_id,
                    wall_float_count: metadata.wall_float_count,
                });
        }
    }

    fn register_collision_edges(&mut self, dat_file: &DatFile) {
        for component in self.component_slots.values() {
            let Some(group_index) = component.group_index() else {
                continue;
            };
            if group_index < 0 {
                continue;
            }

            match component.collision_geometry_kind() {
                CollisionGeometryKind::WallAttributes => {
                    for wall_code in [600_i16, 603_i16] {
                        let Some(edges) =
                            dat_file.visual_collision_edges(
                                group_index as usize,
                                0,
                                wall_code,
                                component.collision_edge_offset(
                                    if wall_code == 600 { 0 } else { 1 },
                                    self.simulation.collision_component_offset,
                                ),
                            )
                        else {
                            continue;
                        };

                        for edge in edges {
                            match edge {
                                VisualCollisionEdge::Line { start, end } => {
                                    self.simulation.edge_manager.add_owned_wall(
                                        crate::engine::physics::EdgeSegment::new(
                                            Vec2::new(start.0, start.1),
                                            Vec2::new(end.0, end.1),
                                        ),
                                        Some(Self::edge_owner_token(
                                            component.id(),
                                            if wall_code == 600 { 0 } else { 1 },
                                        )),
                                    );
                                }
                                VisualCollisionEdge::Circle { center, radius } => {
                                    self.simulation.edge_manager.add_owned_circle(
                                        crate::engine::physics::EdgeCircle::new(
                                            Vec2::new(center.0, center.1),
                                            radius,
                                        ),
                                        Some(Self::edge_owner_token(
                                            component.id(),
                                            if wall_code == 600 { 0 } else { 1 },
                                        )),
                                    );
                                }
                            }
                        }
                    }
                }
                CollisionGeometryKind::OnewayVisual => {
                    let Some(points) = dat_file.visual_primary_points(group_index as usize, 0) else {
                        continue;
                    };
                    if points.len() != 2 {
                        continue;
                    }

                    let point2 = Vec2::new(points[0].0, points[0].1);
                    let point1 = Vec2::new(points[1].0, points[1].1);
                    let solid = crate::engine::physics::EdgeSegment::new(point2, point1)
                        .offset(self.simulation.collision_component_offset);
                    let trigger = crate::engine::physics::EdgeSegment::new(point1, point2)
                        .offset(-self.simulation.collision_component_offset * 0.8);
                    self.simulation
                        .edge_manager
                        .add_owned_wall(solid, Some(Self::edge_owner_token(component.id(), 0)));
                    self.simulation
                        .edge_manager
                        .add_owned_trigger(trigger, Some(Self::edge_owner_token(component.id(), 1)));
                }
                CollisionGeometryKind::VisualCircleAttribute306 => {
                    let Some(VisualCollisionEdge::Circle { center, radius }) =
                        dat_file.visual_circle_attribute_306(group_index as usize, 0)
                    else {
                        continue;
                    };

                    self.simulation.edge_manager.add_owned_circle(
                        crate::engine::physics::EdgeCircle::new(
                            Vec2::new(center.0, center.1),
                            radius,
                        ),
                        Some(Self::edge_owner_token(component.id(), 0)),
                    );
                }
            }
        }
    }

    fn resolve_plunger_position(&mut self, dat_file: &DatFile) {
        let Some(plunger) = self.find_component("plunger") else {
            return;
        };
        let Some(group_index) = plunger.group_index() else {
            return;
        };
        if group_index < 0 {
            return;
        }

        let Some(values) = dat_file.float_attribute(group_index as usize, 0, 601) else {
            return;
        };
        if values.len() >= 2 {
            self.simulation.plunger_position = Vec2::new(values[0], values[1]);
        }
    }

    fn apply_component_float_attributes(&mut self, dat_file: &DatFile) {
        for component in self.component_slots.values_mut() {
            let Some(group_index) = component.group_index() else {
                continue;
            };
            if group_index < 0 {
                continue;
            }

            if let Some(values) = dat_file.float_attribute(group_index as usize, 0, 407) {
                component.apply_float_attribute(407, &values);
            }
        }
    }
}

impl SimulationState {
    const MAX_BALLS: usize = 20;

    pub fn score(&self) -> u64 {
        self.current_player_state().score
    }

    pub fn player_number(&self) -> u8 {
        self.current_player.saturating_add(1)
    }

    pub fn player_count(&self) -> u8 {
        self.player_count.max(1)
    }

    pub fn current_ball_display(&self) -> u8 {
        let current_ball = self
            .max_ball_count
            .saturating_sub(self.current_player_state().ball_count)
            .saturating_add(1);
        current_ball.clamp(1, self.max_ball_count.max(1))
    }

    pub fn set_current_player(&mut self, player_index: u8) {
        let upper_bound = self.player_count().saturating_sub(1);
        self.current_player = player_index.min(upper_bound);
    }

    pub fn start_new_game(&mut self, requested_players: u8) {
        self.player_count = requested_players.clamp(1, PlayerState::MAX_PLAYERS as u8);
        self.current_player = 0;
        self.score_multiplier = 0;
        self.score_added = 0;
        self.bonus_score = 10_000;
        self.bonus_score_active = false;
        self.jackpot_score_active = false;
        self.game_over = false;
        self.game_over_timer_remaining = None;
        self.game_over_ready_for_restart = false;
        for player in &mut self.player_scores {
            *player = PlayerState::new(self.max_ball_count);
        }
        self.pending_messages.clear();
    }

    pub fn reset_player_state(&mut self) {
        self.player_count = 1;
        self.current_player = 0;
        self.score_multiplier = 0;
        self.score_added = 0;
        self.bonus_score = 10_000;
        self.bonus_score_active = false;
        self.jackpot_score_active = false;
        self.game_over = false;
        self.game_over_timer_remaining = None;
        self.game_over_ready_for_restart = false;
        for player in &mut self.player_scores {
            *player = PlayerState::new(self.max_ball_count);
        }
        self.pending_messages.clear();
    }

    pub fn enter_game_over(&mut self) {
        self.game_over = true;
        self.game_over_timer_remaining = Some(3.0);
        self.game_over_ready_for_restart = false;
        self.ball_in_drain = false;
        self.plunger_charge = 0.0;
    }

    pub fn switch_to_next_player(&mut self) -> Option<u8> {
        if self.player_count() <= 1 {
            return None;
        }

        let current = usize::from(self.current_player);
        let player_count = usize::from(self.player_count());
        for offset in 1..=player_count {
            let next = ((current + offset) % player_count) as u8;
            if self.player_scores[usize::from(next)].ball_count > 0 {
                self.current_player = next;
                self.jackpot_score_active = false;
                self.bonus_score_active = false;
                return Some(next);
            }
        }

        None
    }

    pub fn queue_message(&mut self, message: TableMessage) {
        self.pending_messages.push(message);
    }

    pub fn drain_pending_messages(&mut self) -> Vec<TableMessage> {
        std::mem::take(&mut self.pending_messages)
    }

    pub fn resolve_drain_timer(&mut self) -> DrainResolution {
        let current_player = self.current_player;
        let player_count = self.player_count();
        let player = self.current_player_state_mut();
        if player.extra_balls > 0 {
            player.extra_balls -= 1;
            return DrainResolution::ShootAgain;
        }

        player.ball_count = player.ball_count.saturating_sub(1);
        if current_player + 1 != player_count || player.ball_count > 0 {
            DrainResolution::AdvanceTurn
        } else {
            DrainResolution::GameOver
        }
    }

    fn current_player_state(&self) -> &PlayerState {
        &self.player_scores[usize::from(self.current_player)]
    }

    fn current_player_state_mut(&mut self) -> &mut PlayerState {
        &mut self.player_scores[usize::from(self.current_player)]
    }

    pub fn active_ball(&self) -> Option<&Ball> {
        self.balls.first()
    }

    pub fn active_ball_mut(&mut self) -> Option<&mut Ball> {
        self.balls.first_mut()
    }

    pub fn has_active_ball(&self) -> bool {
        !self.balls.is_empty()
    }

    pub fn has_unlaunched_ball(&self) -> bool {
        self.balls.iter().any(|ball| !ball.is_launched())
    }

    pub fn add_ball(&mut self, position: Vec2) -> Option<&mut Ball> {
        if self.balls.len() >= Self::MAX_BALLS {
            return None;
        }

        self.balls.push(Ball::ready_at(position));
        self.sync_ball_counters();
        self.balls.last_mut()
    }

    pub fn ball_count_in_rect(&self, min: Vec2, max: Vec2) -> usize {
        self.balls
            .iter()
            .filter(|ball| {
                ball.position.x >= min.x
                    && ball.position.x <= max.x
                    && ball.position.y >= min.y
                    && ball.position.y <= max.y
            })
            .count()
    }

    pub fn remove_drained_balls(&mut self, drain_y: f32) -> usize {
        let before = self.balls.len();
        self.balls.retain(|ball| !ball.is_drained(drain_y));
        let removed = before.saturating_sub(self.balls.len());
        if removed > 0 {
            self.sync_ball_counters();
        }
        removed
    }

    fn sync_ball_counters(&mut self) {
        self.multiball_count = self.balls.len() as u32;
    }

    pub fn add_score(&mut self, amount: u64) {
        let awarded = self.score_added.saturating_add(
            amount.saturating_mul(Self::score_multiplier_value(self.score_multiplier)),
        );
        if self.jackpot_score_active {
            let jackpot_limit = 5_000_000_u64;
            let jackpot = &mut self.current_player_state_mut().jackpot_score;
            *jackpot = jackpot.saturating_add(amount).min(jackpot_limit);
        }
        if self.bonus_score_active {
            self.bonus_score = self.bonus_score.saturating_add(amount).min(5_000_000);
        }
        let player = self.current_player_state_mut();
        player.score = player.score.saturating_add(awarded);
        if player.score >= 1_000_000_000 {
            player.score_e9_part = player
                .score_e9_part
                .saturating_add((player.score / 1_000_000_000) as u32);
            player.score %= 1_000_000_000;
        }
    }

    pub fn special_add_score(&mut self, amount: u64) -> u64 {
        let bonus_active = self.bonus_score_active;
        let jackpot_active = self.jackpot_score_active;
        let score_multiplier = self.score_multiplier;
        let score_added = self.score_added;

        self.bonus_score_active = false;
        self.jackpot_score_active = false;
        self.score_multiplier = 0;
        self.score_added = 0;

        let score_before = self.score();
        let e9_before = self.current_player_state().score_e9_part;
        self.add_score(amount);
        let score_after = self.score();
        let e9_after = self.current_player_state().score_e9_part;

        self.bonus_score_active = bonus_active;
        self.jackpot_score_active = jackpot_active;
        self.score_multiplier = score_multiplier;
        self.score_added = score_added;

        u64::from(e9_after.saturating_sub(e9_before)) * 1_000_000_000
            + score_after.saturating_sub(score_before)
    }

    fn update_activity_state(&mut self, dt: f32) {
        let decay = (1.0 - dt.max(0.0) * 1.6).clamp(0.0, 1.0);
        self.activities.ramp_activity *= decay;
        self.activities.lower_hazard_activity *= decay;
        self.activities.orbit_activity *= decay;
        self.activities.target_activity *= decay;
        self.activities.bumper_activity *= decay;
        self.activities.lane_activity *= decay;

        if let Some(ball) = self.active_ball() {
            let ball_x = (ball.position.x / 600.0).clamp(0.0, 1.0);
            let ball_y = (ball.position.y / 416.0).clamp(0.0, 1.0);
            let top = (1.0 - ball_y).clamp(0.0, 1.0);
            let right = ball_x;
            let left = (1.0 - ball_x).clamp(0.0, 1.0);
            let horizontal_speed = (ball.velocity.x.abs() / 700.0).clamp(0.0, 1.0);
            let total_speed =
                ((ball.velocity.x.abs() + ball.velocity.y.abs()) / 900.0).clamp(0.0, 1.0);
            let bumper_presence = ((top * 0.25)
                + (ball_y * 0.15)
                + (ball.velocity.y.abs() / 700.0 * 0.30)
                + (total_speed * 0.30))
                .clamp(0.0, 1.0);
            let lane_presence = ((ball_x * 0.15)
                + (top * 0.40)
                + (self.plunger_charge * 0.25)
                + (self.regions.lane_ready * 0.20))
                .clamp(0.0, 1.0);
            let ramp_presence = ((ball.position.x / 600.0).clamp(0.0, 1.0) * 0.55
                + (1.0 - (ball.position.y / 416.0)).clamp(0.0, 1.0) * 0.45)
                .clamp(0.0, 1.0);
            let lower_hazard_presence = (((ball.position.y / 416.0).clamp(0.0, 1.0) * 0.60)
                + ((1.0 - (ball.position.x / 600.0).clamp(0.0, 1.0)) * 0.25)
                + ((ball.velocity.y.max(0.0) / 600.0).clamp(0.0, 1.0) * 0.15))
                .clamp(0.0, 1.0);
            let orbit_presence =
                ((top * 0.45) + (right * 0.35) + (horizontal_speed * 0.20)).clamp(0.0, 1.0);
            let target_presence = ((top * 0.35)
                + (right.max(left) * 0.20)
                + (horizontal_speed * 0.20)
                + (total_speed * 0.25))
                .clamp(0.0, 1.0);

            self.activities.ramp_activity = self.activities.ramp_activity.max(ramp_presence);
            self.activities.lower_hazard_activity = self
                .activities
                .lower_hazard_activity
                .max(lower_hazard_presence);
            self.activities.orbit_activity = self.activities.orbit_activity.max(orbit_presence);
            self.activities.target_activity = self.activities.target_activity.max(target_presence);
            self.activities.bumper_activity = self.activities.bumper_activity.max(bumper_presence);
            self.activities.lane_activity = self.activities.lane_activity.max(lane_presence);
        }

        if self.drain_count > self.previous_drain_count {
            self.activities.lower_hazard_activity = 1.0;
        }

        if self.launch_count > self.previous_launch_count {
            self.activities.ramp_activity = self.activities.ramp_activity.max(0.75);
            self.activities.orbit_activity = self.activities.orbit_activity.max(0.70);
            self.activities.target_activity = self.activities.target_activity.max(0.55);
            self.activities.bumper_activity = self.activities.bumper_activity.max(0.60);
            self.activities.lane_activity = self.activities.lane_activity.max(0.80);
        }
    }

    fn refresh_derived_state(&mut self) {
        let launch_progress = (self.launch_count.min(6) as f32 / 6.0).clamp(0.0, 1.0);
        let drain_progress = (self.drain_count.min(6) as f32 / 6.0).clamp(0.0, 1.0);
        let score_progress = (self.score().min(8_000) as f32 / 8_000.0).clamp(0.0, 1.0);

        self.regions = self
            .active_ball()
            .map(|ball| TableRegionState {
                lane_ready: if ball.is_launched() {
                    self.plunger_charge
                } else {
                    1.0
                },
                ball_x: (ball.position.x / 600.0).clamp(0.0, 1.0),
                ball_y: (1.0 - (ball.position.y / 416.0)).clamp(0.0, 1.0),
                left: (1.0 - (ball.position.x / 600.0).clamp(0.0, 1.0)).clamp(0.0, 1.0),
                right: (ball.position.x / 600.0).clamp(0.0, 1.0),
                top: (1.0 - (ball.position.y / 416.0)).clamp(0.0, 1.0),
                bottom: (ball.position.y / 416.0).clamp(0.0, 1.0),
                ramp: (((ball.position.x / 600.0).clamp(0.0, 1.0) * 0.55)
                    + ((1.0 - (ball.position.y / 416.0)).clamp(0.0, 1.0) * 0.30)
                    + (launch_progress * 0.15))
                    .clamp(0.0, 1.0),
            })
            .unwrap_or(TableRegionState {
                lane_ready: self.plunger_charge,
                ball_x: 0.0,
                ball_y: 0.0,
                left: 1.0,
                right: 0.0,
                top: 0.0,
                bottom: 1.0,
                ramp: (launch_progress * 0.15).clamp(0.0, 1.0),
            });

        self.visual_signals = TableVisualSignalState {
            score_progress,
            launch_progress,
            drain_progress,
            impact_focus: ((score_progress * 0.45)
                + (self.plunger_charge * 0.35)
                + (launch_progress * 0.20))
                .clamp(0.0, 1.0),
            recovery_focus: ((drain_progress * 0.45)
                + (launch_progress * 0.35)
                + (self.regions.lane_ready * 0.20))
                .clamp(0.0, 1.0),
            lane_focus: ((self.regions.lane_ready * 0.45)
                + (launch_progress * 0.35)
                + (score_progress * 0.20))
                .clamp(0.0, 1.0),
            hazard_focus: ((drain_progress * 0.50)
                + (self.plunger_charge * 0.30)
                + (score_progress * 0.20))
                .clamp(0.0, 1.0),
            target_focus: ((score_progress * 0.55)
                + (launch_progress * 0.25)
                + (drain_progress * 0.20))
                .clamp(0.0, 1.0),
            orbit_focus: ((launch_progress * 0.45)
                + (score_progress * 0.35)
                + (self.plunger_charge * 0.20))
                .clamp(0.0, 1.0),
            field_light_focus: ((self.regions.ball_x * 0.35)
                + (self.regions.ball_y * 0.25)
                + (score_progress * 0.20)
                + (launch_progress * 0.20))
                .clamp(0.0, 1.0),
            rollover_light_focus: ((self.regions.ball_y * 0.40)
                + (self.regions.lane_ready * 0.20)
                + (launch_progress * 0.20)
                + (score_progress * 0.20))
                .clamp(0.0, 1.0),
            fuel_focus: ((self.plunger_charge * 0.45)
                + (self.regions.lane_ready * 0.30)
                + (self.regions.ball_x * 0.25))
                .clamp(0.0, 1.0),
        };
    }

    pub fn game_over_ready_for_restart(&self) -> bool {
        self.game_over_ready_for_restart
    }

    pub fn jackpot_score(&self) -> u64 {
        self.current_player_state().jackpot_score
    }

    const fn score_multiplier_value(index: u8) -> u64 {
        match index {
            0 => 1,
            1 => 2,
            2 => 3,
            3 => 5,
            _ => 10,
        }
    }

    fn tick_table_timers(&mut self, dt: f32) {
        if let Some(timer_remaining) = self.game_over_timer_remaining.as_mut() {
            *timer_remaining -= dt.max(0.0);
            if *timer_remaining <= 0.0 {
                self.game_over_timer_remaining = None;
                self.game_over_ready_for_restart = true;
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrainResolution {
    ShootAgain,
    AdvanceTurn,
    GameOver,
}

#[cfg(test)]
mod tests {
    use super::{PinballTable, SimulationState};
    use crate::engine::math::Vec2;
    use crate::gameplay::components::{ComponentId, MessageCode, TableMessage};
    use crate::gameplay::mechanics::{DrainMechanic, PlungerMechanic};

    #[test]
    fn ball_count_in_rect_counts_multiple_active_balls() {
        let mut simulation = SimulationState::default();
        let _ = simulation.add_ball(Vec2::new(100.0, 100.0));
        let _ = simulation.add_ball(Vec2::new(120.0, 120.0));
        let _ = simulation.add_ball(Vec2::new(300.0, 300.0));

        assert_eq!(
            simulation.ball_count_in_rect(Vec2::new(90.0, 90.0), Vec2::new(150.0, 150.0)),
            2
        );
        assert_eq!(
            simulation.ball_count_in_rect(Vec2::new(250.0, 250.0), Vec2::new(350.0, 350.0)),
            1
        );
    }

    #[test]
    fn add_score_tracks_current_player_only() {
        let mut simulation = SimulationState::default();
        simulation.start_new_game(2);
        simulation.add_score(1_200);

        assert_eq!(simulation.score(), 1_200);
        assert_eq!(simulation.player_scores[0].score, 1_200);
        assert_eq!(simulation.player_scores[1].score, 0);

        simulation.set_current_player(1);
        simulation.add_score(500);

        assert_eq!(simulation.score(), 500);
        assert_eq!(simulation.player_scores[0].score, 1_200);
        assert_eq!(simulation.player_scores[1].score, 500);
    }

    #[test]
    fn add_score_applies_source_shaped_multiplier_and_base_bonus() {
        let mut simulation = SimulationState::default();
        simulation.score_multiplier = 3;
        simulation.score_added = 25;

        simulation.add_score(100);

        assert_eq!(simulation.score(), 525);
    }

    #[test]
    fn add_score_updates_bonus_and_jackpot_accumulators_when_active() {
        let mut simulation = SimulationState::default();
        simulation.player_scores[0].jackpot_score = 20_000;
        simulation.bonus_score_active = true;
        simulation.jackpot_score_active = true;

        simulation.add_score(3_500);

        assert_eq!(simulation.score(), 3_500);
        assert_eq!(simulation.bonus_score, 13_500);
        assert_eq!(simulation.jackpot_score(), 23_500);
    }

    #[test]
    fn add_score_caps_bonus_and_jackpot_accumulators_like_source() {
        let mut simulation = SimulationState::default();
        simulation.bonus_score = 4_999_000;
        simulation.player_scores[0].jackpot_score = 4_999_000;
        simulation.bonus_score_active = true;
        simulation.jackpot_score_active = true;

        simulation.add_score(10_000);

        assert_eq!(simulation.bonus_score, 5_000_000);
        assert_eq!(simulation.jackpot_score(), 5_000_000);
    }

    #[test]
    fn special_add_score_temporarily_ignores_multiplier_and_accumulator_flags() {
        let mut simulation = SimulationState::default();
        simulation.score_multiplier = 4;
        simulation.score_added = 50;
        simulation.bonus_score_active = true;
        simulation.jackpot_score_active = true;
        simulation.bonus_score = 10_000;
        simulation.player_scores[0].jackpot_score = 20_000;

        let awarded = simulation.special_add_score(1_000);

        assert_eq!(awarded, 1_000);
        assert_eq!(simulation.score(), 1_000);
        assert_eq!(simulation.bonus_score, 10_000);
        assert_eq!(simulation.jackpot_score(), 20_000);
        assert!(simulation.bonus_score_active);
        assert!(simulation.jackpot_score_active);
        assert_eq!(simulation.score_multiplier, 4);
    }

    #[test]
    fn switch_to_next_player_restores_current_player_score() {
        let mut table = PinballTable::default();
        table.dispatch(TableMessage::with_value(MessageCode::NewGame, 2.0));
        table.simulation.add_score(1_000);

        table.dispatch(TableMessage::from_code(MessageCode::SwitchToNextPlayer));
        assert_eq!(table.simulation.current_player, 1);
        assert_eq!(table.score(), 0);
        assert!(!table.simulation.bonus_score_active);
        assert!(!table.simulation.jackpot_score_active);

        table.simulation.add_score(250);
        table.dispatch(TableMessage::from_code(MessageCode::SwitchToNextPlayer));

        assert_eq!(table.simulation.current_player, 0);
        assert_eq!(table.score(), 1_000);
        assert_eq!(table.simulation.player_scores[1].score, 250);
        assert!(
            table
                .message_log()
                .iter()
                .any(|message| *message == TableMessage::with_value(MessageCode::PlayerChanged, 1.0))
        );
    }

    #[test]
    fn switch_to_next_player_restores_per_player_jackpot_state() {
        let mut table = PinballTable::default();
        table.dispatch(TableMessage::with_value(MessageCode::NewGame, 2.0));
        table.simulation.player_scores[0].jackpot_score = 75_000;
        table.simulation.player_scores[1].jackpot_score = 125_000;
        table.simulation.jackpot_score_active = true;
        table.simulation.bonus_score_active = true;

        table.dispatch(TableMessage::from_code(MessageCode::SwitchToNextPlayer));

        assert_eq!(table.simulation.current_player, 1);
        assert_eq!(table.simulation.jackpot_score(), 125_000);
        assert!(!table.simulation.jackpot_score_active);
        assert!(!table.simulation.bonus_score_active);

        table.dispatch(TableMessage::from_code(MessageCode::SwitchToNextPlayer));
        assert_eq!(table.simulation.current_player, 0);
        assert_eq!(table.simulation.jackpot_score(), 75_000);
    }

    #[test]
    fn drain_timer_expiry_decrements_ball_count_and_switches_player() {
        let mut table = PinballTable::default();
        table.add_component(DrainMechanic::new(ComponentId(1), "drain"));
        table.add_component(PlungerMechanic::new(ComponentId(2), "plunger"));
        table.simulation.start_new_game(2);
        let _ = table.simulation.add_ball(Vec2::new(100.0, 420.0));

        table.tick_components(0.0);
        assert!(table.simulation.ball_in_drain);
        assert_eq!(table.simulation.player_scores[0].ball_count, 3);

        table.tick_components(1.0);

        assert_eq!(table.simulation.player_scores[0].ball_count, 2);
        assert_eq!(table.simulation.current_player, 1);
        assert!(
            table
                .message_log()
                .iter()
                .any(|message| *message == TableMessage::with_value(MessageCode::PlayerChanged, 1.0))
        );
        assert!(
            table
                .message_log()
                .iter()
                .any(|message| *message == TableMessage::from_code(MessageCode::PlungerStartFeedTimer))
        );

        table.tick_components(0.96);
        assert!(table.simulation.has_unlaunched_ball());
    }

    #[test]
    fn drain_timer_expiry_uses_extra_ball_before_switching_player() {
        let mut table = PinballTable::default();
        table.add_component(DrainMechanic::new(ComponentId(1), "drain"));
        table.add_component(PlungerMechanic::new(ComponentId(2), "plunger"));
        table.simulation.start_new_game(2);
        table.simulation.player_scores[0].extra_balls = 1;
        let _ = table.simulation.add_ball(Vec2::new(100.0, 420.0));

        table.tick_components(0.0);
        table.tick_components(1.0);

        assert_eq!(table.simulation.current_player, 0);
        assert_eq!(table.simulation.player_scores[0].extra_balls, 0);
        assert_eq!(table.simulation.player_scores[0].ball_count, 3);
        assert!(
            !table
                .message_log()
                .iter()
                .any(|message| matches!(message, TableMessage::Code(MessageCode::PlayerChanged, _)))
        );

        table.tick_components(0.96);
        assert!(table.simulation.has_unlaunched_ball());
    }

    #[test]
    fn game_over_message_sets_endgame_state_until_restart_window() {
        let mut table = PinballTable::default();

        table.dispatch(TableMessage::from_code(MessageCode::GameOver));
        table.refresh_text_boxes();
        assert!(table.simulation.game_over);
        assert!(!table.simulation.game_over_ready_for_restart());
        assert_eq!(
            table.simulation.info_box.current_text(),
            Some("GAME OVER")
        );

        table.tick_components(3.0);
        table.refresh_text_boxes();
        assert!(table.simulation.game_over_ready_for_restart());
        assert_eq!(
            table.simulation.info_box.current_text(),
            Some("PRESS START FOR NEW GAME")
        );
    }

    #[test]
    fn last_ball_drain_transitions_table_into_game_over_state() {
        let mut table = PinballTable::default();
        table.add_component(DrainMechanic::new(ComponentId(1), "drain"));
        table.simulation.player_scores[0].ball_count = 1;
        let _ = table.simulation.add_ball(Vec2::new(100.0, 420.0));

        table.tick_components(0.0);
        table.tick_components(1.0);

        assert!(table.simulation.game_over);
        assert_eq!(table.simulation.player_scores[0].ball_count, 0);
        assert!(
            table
                .message_log()
                .iter()
                .any(|message| *message == TableMessage::from_code(MessageCode::GameOver))
        );
    }
}
