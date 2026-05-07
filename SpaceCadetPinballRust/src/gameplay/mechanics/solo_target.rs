use crate::engine::physics::{CollisionContact, CollisionEdgeRole};
use crate::gameplay::components::{
    ComponentId, ComponentState, GameplayComponent, MessageCode, SimulationState, TableInputState,
    TableMessage,
};

pub struct SoloTargetMechanic {
    state: ComponentState,
    timer_remaining: Option<f32>,
    timer_time: f32,
}

impl SoloTargetMechanic {
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
        }
    }

    fn apply_active_visual(&mut self) {
        self.state.sprite_index = if self.state.active { 0 } else { 1 };
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
            TableMessage::Code(MessageCode::Reset, _) => {
                self.timer_remaining = None;
                self.state.active = true;
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
        simulation.add_score(self.state.collision_score());
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
}
