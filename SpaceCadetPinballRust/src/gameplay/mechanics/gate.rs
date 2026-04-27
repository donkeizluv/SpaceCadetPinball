use crate::gameplay::components::{
    ComponentId, ComponentState, GameplayComponent, MessageCode, SimulationState, TableInputState,
    TableMessage,
};

pub struct GateMechanic {
    state: ComponentState,
}

impl GateMechanic {
    pub fn new(id: ComponentId, name: &'static str) -> Self {
        Self::from_state(ComponentState::new(id, name).with_control("GateControl"))
    }

    pub fn from_state(mut state: ComponentState) -> Self {
        state.active = true;
        state.sprite_index = 0;
        Self { state }
    }
}

impl GameplayComponent for GateMechanic {
    fn state(&self) -> &ComponentState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ComponentState {
        &mut self.state
    }

    fn collision_edge_offset(&self, slot: u8, collision_component_offset: f32) -> f32 {
        if slot == 0 {
            collision_component_offset
        } else {
            0.0
        }
    }

    fn on_message(
        &mut self,
        message: TableMessage,
        _simulation: &mut SimulationState,
        _table_state: &TableInputState,
    ) {
        match message {
            TableMessage::Code(MessageCode::TGateDisable, _) => {
                self.state.active = false;
                self.state.sprite_index = -1;
            }
            TableMessage::Code(MessageCode::Reset | MessageCode::TGateEnable, _) => {
                self.state.active = true;
                self.state.sprite_index = 0;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::gameplay::components::TableMessage;

    use super::*;

    #[test]
    fn gate_messages_match_enable_disable_reset_behavior() {
        let mut gate = GateMechanic::new(ComponentId(1), "v_gate1");
        let mut simulation = SimulationState::default();
        let table_state = TableInputState::default();

        assert!(gate.state.active);
        assert_eq!(gate.state.sprite_index, 0);

        gate.on_message(
            TableMessage::from_code(MessageCode::TGateDisable),
            &mut simulation,
            &table_state,
        );
        assert!(!gate.state.active);
        assert_eq!(gate.state.sprite_index, -1);

        gate.on_message(
            TableMessage::from_code(MessageCode::Reset),
            &mut simulation,
            &table_state,
        );
        assert!(gate.state.active);
        assert_eq!(gate.state.sprite_index, 0);
    }
}
