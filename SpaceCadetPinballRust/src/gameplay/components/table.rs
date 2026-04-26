mod input;
mod text_box;
mod visuals;

use std::collections::HashMap;

use crate::engine::TableBridgeState;
use crate::engine::physics::{Ball, EdgeManager};
use crate::gameplay::mechanics::{DrainMechanic, FlipperMechanic, PlungerMechanic};

use super::component::GameplayComponent;
use super::group::{ComponentGroup, ComponentId};
use super::messages::TableMessage;

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
}

pub struct SimulationState {
    pub ball: Option<Ball>,
    pub plunger_charge: f32,
    pub launch_count: u64,
    pub drain_count: u64,
    pub left_flipper_active: bool,
    pub right_flipper_active: bool,
    pub edge_manager: EdgeManager,
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
            plunger_charge: 0.0,
            launch_count: 0,
            drain_count: 0,
            left_flipper_active: false,
            right_flipper_active: false,
            edge_manager: EdgeManager::for_table_bounds(600.0, 416.0),
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
        }
    }
}

impl PinballTable {
    pub fn new() -> Self {
        let mut table = Self::default();
        table.add_component(FlipperMechanic::new(ComponentId(1), "flipper"));
        table.add_component(PlungerMechanic::new(ComponentId(2), "plunger"));
        table.add_component(DrainMechanic::new(ComponentId(3), "drain"));
        table.refresh_text_boxes();
        table
    }

    pub fn components(&self) -> &ComponentGroup {
        &self.components
    }

    pub fn register_component(&mut self, id: ComponentId, name: impl Into<String>) {
        self.components.register(id, name);
    }

    pub fn add_component(&mut self, component: impl GameplayComponent + 'static) {
        let id = component.id();
        let name = component.name().to_string();
        self.components.register(id, name);
        self.component_slots.insert(id, Box::new(component));
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

    pub fn component_count(&self) -> usize {
        self.component_slots.len()
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
        }

        self.message_log.push(message);
        self.broadcast_message(message);
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
    }

    pub fn step_simulation(&mut self, dt: f32) {
        if let Some(ball) = self.simulation.ball.as_mut() {
            if let Some(nudge) = self.input_state.pending_nudge.take() {
                ball.apply_nudge(nudge);
            }
            ball.step(dt);
            self.simulation.edge_manager.set_flipper_state(
                self.simulation.left_flipper_active,
                self.simulation.right_flipper_active,
            );
            let _ = self.simulation.edge_manager.resolve_ball(ball);
        }

        self.input_state.pending_start = false;
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
}
