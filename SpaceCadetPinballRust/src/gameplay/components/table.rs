mod builder;
mod input;
mod text_box;
mod visuals;

use std::collections::HashMap;

use crate::assets::{DatFile, VisualCollisionEdge};
use crate::engine::TableBridgeState;
use crate::engine::math::Vec2;
use crate::engine::physics::{
    Ball, CollisionComponentMetadata, CollisionComponentRegistry, EdgeManager,
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
    pub ball: Option<Ball>,
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
    previous_ball_present: bool,
    previous_launch_count: u64,
    previous_drain_count: u64,
}

impl Default for SimulationState {
    fn default() -> Self {
        Self {
            ball: None,
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
        self.simulation.ball.as_ref()
    }

    pub fn launch_count(&self) -> u64 {
        self.simulation.launch_count
    }

    pub fn drain_count(&self) -> u64 {
        self.simulation.drain_count
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

        self.message_log.push(message);
        self.broadcast_message(message);
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
        self.simulation.refresh_derived_state();
    }

    pub fn step_simulation(&mut self, dt: f32) {
        if let Some(mut ball) = self.simulation.ball.take() {
            if let Some(nudge) = self.input_state.pending_nudge.take() {
                ball.apply_nudge(nudge);
            }
            ball.step(dt);
            self.simulation.edge_manager.set_flipper_state(
                self.simulation.left_flipper_active,
                self.simulation.right_flipper_active,
            );
            let _ = self
                .simulation
                .edge_manager
                .resolve_ball_with_filter(&mut ball, |owner_token| match owner_token {
                    Some(owner_token) => {
                        let (component_id, slot) = Self::decode_edge_owner_token(owner_token);
                        self.component_slots
                            .get(&component_id)
                            .is_none_or(|component| component.collision_edge_active(slot))
                    }
                    None => true,
                });
            self.simulation.ball = Some(ball);
        }

        self.input_state.pending_start = false;
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
}

impl SimulationState {
    fn update_activity_state(&mut self, dt: f32) {
        let decay = (1.0 - dt.max(0.0) * 1.6).clamp(0.0, 1.0);
        self.activities.ramp_activity *= decay;
        self.activities.lower_hazard_activity *= decay;
        self.activities.orbit_activity *= decay;
        self.activities.target_activity *= decay;
        self.activities.bumper_activity *= decay;
        self.activities.lane_activity *= decay;

        if let Some(ball) = self.ball.as_ref() {
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
        let score_value = self
            .launch_count
            .saturating_mul(1000)
            .saturating_add(self.drain_count.saturating_mul(500));
        let score_progress = (score_value.min(8_000) as f32 / 8_000.0).clamp(0.0, 1.0);

        self.regions = self
            .ball
            .as_ref()
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
}
