use crate::gameplay::components::{
    CollisionGeometryKind, ComponentId, ComponentState, GameplayComponent, MessageCode,
    SimulationState, TableInputState, TableMessage,
};
use crate::engine::physics::{CollisionContact, CollisionEdgeRole};

pub struct OnewayMechanic {
    state: ComponentState,
    trigger_count: u32,
}

impl OnewayMechanic {
    pub fn new(id: ComponentId, name: &'static str) -> Self {
        Self::from_state(ComponentState::new(id, name).with_control("OnewayControl"))
    }

    pub fn from_state(mut state: ComponentState) -> Self {
        state.active = true;
        Self {
            state,
            trigger_count: 0,
        }
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
        if let TableMessage::Code(code, _) = message {
            match code {
                MessageCode::Reset => {
                    self.state.active = true;
                    self.trigger_count = 0;
                }
                MessageCode::ControlCollision => {
                    self.trigger_count = self.trigger_count.saturating_add(1);
                }
                _ => {}
            }
        }
    }

    fn on_collision(
        &mut self,
        slot: u8,
        edge_role: CollisionEdgeRole,
        _contact: CollisionContact,
        simulation: &mut SimulationState,
        _table_state: &TableInputState,
    ) {
        if simulation.tilt_locked {
            return;
        }

        if slot == 1 && edge_role == CollisionEdgeRole::Trigger {
            self.trigger_count = self.trigger_count.saturating_add(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::physics::{CollisionContact, CollisionEdgeRole};
    use crate::gameplay::components::{GameplayComponent, TableMessage};

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

        oneway.on_message(
            TableMessage::from_code(MessageCode::ControlCollision),
            &mut simulation,
            &table_state,
        );
        assert_eq!(oneway.trigger_count, 1);

        oneway.on_collision(
            0,
            CollisionEdgeRole::Solid,
            CollisionContact::new(crate::engine::math::Vec2::ZERO, crate::engine::math::Vec2::ZERO, 0.0),
            &mut simulation,
            &table_state,
        );
        assert_eq!(oneway.trigger_count, 1);

        oneway.on_collision(
            1,
            CollisionEdgeRole::Trigger,
            CollisionContact::new(crate::engine::math::Vec2::ZERO, crate::engine::math::Vec2::ZERO, 0.0),
            &mut simulation,
            &table_state,
        );
        assert_eq!(oneway.trigger_count, 2);
    }
}
