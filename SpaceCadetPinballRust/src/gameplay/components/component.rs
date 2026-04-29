use crate::engine::physics::{CollisionContact, CollisionEdgeRole};

use super::group::ComponentId;
use super::messages::TableMessage;
use super::table::{SimulationState, TableInputState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionGeometryKind {
    WallAttributes,
    OnewayVisual,
    VisualCircleAttribute306,
}

#[derive(Debug, Clone)]
pub struct ComponentState {
    pub id: ComponentId,
    pub active: bool,
    pub message_field: i32,
    pub sprite_index: i32,
    pub group_name: String,
    pub group_index: Option<i32>,
    pub control_name: Option<&'static str>,
    pub scoring: Vec<i32>,
}

impl ComponentState {
    pub fn new(id: ComponentId, group_name: impl Into<String>) -> Self {
        Self {
            id,
            active: true,
            message_field: 0,
            sprite_index: -1,
            group_name: group_name.into(),
            group_index: None,
            control_name: None,
            scoring: Vec::new(),
        }
    }

    pub fn with_group_index(mut self, group_index: i32) -> Self {
        self.group_index = Some(group_index);
        self
    }

    pub fn with_control(mut self, control_name: &'static str) -> Self {
        self.control_name = Some(control_name);
        self
    }

    pub fn with_scoring(mut self, scoring: impl Into<Vec<i32>>) -> Self {
        self.scoring = scoring.into();
        self
    }

    pub fn collision_score(&self) -> u64 {
        self.scoring
            .first()
            .copied()
            .unwrap_or(0)
            .max(0) as u64
    }
}

pub trait GameplayComponent {
    fn state(&self) -> &ComponentState;

    fn state_mut(&mut self) -> &mut ComponentState;

    fn id(&self) -> ComponentId {
        self.state().id
    }

    fn name(&self) -> &str {
        &self.state().group_name
    }

    fn group_index(&self) -> Option<i32> {
        self.state().group_index
    }

    fn is_active(&self) -> bool {
        self.state().active
    }

    fn collision_geometry_kind(&self) -> CollisionGeometryKind {
        CollisionGeometryKind::WallAttributes
    }

    fn collision_edge_active(&self, _slot: u8) -> bool {
        self.is_active()
    }

    fn collision_edge_offset(&self, _slot: u8, _collision_component_offset: f32) -> f32 {
        0.0
    }

    fn apply_float_attribute(&mut self, _attribute_id: i16, _values: &[f32]) {}

    fn on_message(
        &mut self,
        message: TableMessage,
        simulation: &mut SimulationState,
        table_state: &TableInputState,
    );

    fn tick(
        &mut self,
        _simulation: &mut SimulationState,
        _table_state: &TableInputState,
        _dt: f32,
    ) {
    }

    fn on_collision(
        &mut self,
        _slot: u8,
        edge_role: CollisionEdgeRole,
        contact: CollisionContact,
        simulation: &mut SimulationState,
        table_state: &TableInputState,
    ) {
        if edge_role == CollisionEdgeRole::Trigger || contact.threshold_exceeded {
            self.on_message(
                TableMessage::from_code(super::messages::MessageCode::ControlCollision),
                simulation,
                table_state,
            );
        }
    }
}
