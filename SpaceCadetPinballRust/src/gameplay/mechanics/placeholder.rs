use crate::gameplay::components::{
    ComponentState, GameplayComponent, MessageCode, SimulationState, TableInputState, TableMessage,
};

pub struct PlaceholderMechanic {
    state: ComponentState,
}

impl PlaceholderMechanic {
    pub fn from_state(state: ComponentState) -> Self {
        Self { state }
    }
}

impl GameplayComponent for PlaceholderMechanic {
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
            TableMessage::Code(
                MessageCode::TBlockerDisable
                | MessageCode::TGateDisable
                | MessageCode::TPopupTargetDisable
                | MessageCode::TSoloTargetDisable,
                _,
            ) => {
                self.state.active = false;
            }
            TableMessage::Code(
                MessageCode::TBlockerEnable
                | MessageCode::TGateEnable
                | MessageCode::TPopupTargetEnable
                | MessageCode::TSoloTargetEnable,
                _,
            ) => {
                self.state.active = true;
            }
            TableMessage::Code(MessageCode::Reset, _) => {
                self.state.active = true;
                self.state.message_field = 0;
            }
            TableMessage::Code(MessageCode::TLightSetMessageField, value) => {
                self.state.message_field = value as i32;
            }
            _ => {}
        }
    }
}
