use crate::gameplay::components::{
    CollisionGeometryKind, ComponentId, ComponentState, GameplayComponent, MessageCode,
    SimulationState, TableInputState, TableMessage,
};

pub struct OnewayMechanic {
    state: ComponentState,
}

impl OnewayMechanic {
    pub fn new(id: ComponentId, name: &'static str) -> Self {
        Self::from_state(ComponentState::new(id, name).with_control("OnewayControl"))
    }

    pub fn from_state(mut state: ComponentState) -> Self {
        state.active = true;
        Self { state }
    }
}

impl GameplayComponent for OnewayMechanic {
    fn state(&self) -> &ComponentState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ComponentState {
        &mut self.state
    }

    fn collision_geometry_kind(&self) -> CollisionGeometryKind {
        CollisionGeometryKind::OnewayVisual
    }

    fn on_message(
        &mut self,
        message: TableMessage,
        _simulation: &mut SimulationState,
        _table_state: &TableInputState,
    ) {
        if let TableMessage::Code(MessageCode::Reset, _) = message {
            self.state.active = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::gameplay::components::TableMessage;

    use super::*;

    #[test]
    fn oneway_uses_visual_geometry_registration_path() {
        let mut oneway = OnewayMechanic::new(ComponentId(1), "s_onewy1");
        let mut simulation = SimulationState::default();
        let table_state = TableInputState::default();

        assert_eq!(
            oneway.collision_geometry_kind(),
            CollisionGeometryKind::OnewayVisual
        );

        oneway.on_message(
            TableMessage::from_code(MessageCode::Reset),
            &mut simulation,
            &table_state,
        );
        assert!(oneway.state.active);
    }
}
