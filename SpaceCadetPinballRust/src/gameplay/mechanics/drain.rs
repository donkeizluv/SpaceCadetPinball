use crate::gameplay::components::{
    ComponentId, ComponentState, DrainResolution, GameplayComponent, MessageCode,
    SimulationState, TableInputState, TableMessage,
};

pub struct DrainMechanic {
    state: ComponentState,
    drain_y: f32,
    timer_remaining: Option<f32>,
    timer_time: f32,
}

impl DrainMechanic {
    pub fn new(id: ComponentId, name: &'static str) -> Self {
        Self::from_state(ComponentState::new(id, name).with_control("BallDrainControl"))
    }

    pub fn from_state(state: ComponentState) -> Self {
        Self {
            state,
            drain_y: 408.0,
            timer_remaining: None,
            timer_time: 1.0,
        }
    }
}

impl GameplayComponent for DrainMechanic {
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
        if let TableMessage::Code(MessageCode::Reset, _) = message {
            self.timer_remaining = None;
        }
    }

    fn tick(&mut self, simulation: &mut SimulationState, _table_state: &TableInputState, dt: f32) {
        let drained_count = simulation.remove_drained_balls(self.drain_y);
        if drained_count > 0 {
            simulation.drain_count = simulation.drain_count.saturating_add(drained_count as u64);
            if simulation.multiball_count == 0 {
                if !simulation.ball_in_drain && !simulation.tilt_locked {
                    let _ = simulation.special_add_score(simulation.bonus_score);
                }
                simulation.ball_in_drain = true;
                self.timer_remaining = Some(self.timer_time);
            }
        }

        if let Some(timer) = self.timer_remaining.as_mut() {
            *timer -= dt.max(0.0);
            if *timer <= 0.0 {
                self.timer_remaining = None;
                match simulation.resolve_drain_timer() {
                    DrainResolution::AdvanceTurn => {
                        simulation.queue_message(TableMessage::from_code(
                            MessageCode::SwitchToNextPlayer,
                        ));
                        simulation.queue_message(TableMessage::from_code(
                            MessageCode::PlungerStartFeedTimer,
                        ));
                    }
                    DrainResolution::ShootAgain => {
                        simulation.queue_message(TableMessage::from_code(
                            MessageCode::PlungerStartFeedTimer,
                        ));
                    }
                    DrainResolution::GameOver => {
                        simulation.queue_message(TableMessage::from_code(MessageCode::GameOver));
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::math::Vec2;
    use crate::gameplay::components::TableMessage;

    use super::*;

    #[test]
    fn drain_sets_ball_in_drain_and_timer_when_last_ball_drains() {
        let mut drain = DrainMechanic::new(ComponentId(1), "drain");
        let mut simulation = SimulationState::default();
        let table_state = TableInputState::default();
        let _ = simulation.add_ball(Vec2::new(100.0, 420.0));

        drain.tick(&mut simulation, &table_state, 0.0);
        assert!(!simulation.has_active_ball());
        assert!(simulation.ball_in_drain);
        assert_eq!(simulation.multiball_count, 0);
        assert_eq!(simulation.drain_count, 1);
        assert_eq!(drain.timer_remaining, Some(1.0));

        drain.on_message(
            TableMessage::from_code(MessageCode::Reset),
            &mut simulation,
            &table_state,
        );
        assert_eq!(drain.timer_remaining, None);
    }

    #[test]
    fn drain_timer_resolution_decrements_ball_count_then_requests_turn_advance() {
        let mut drain = DrainMechanic::new(ComponentId(1), "drain");
        let mut simulation = SimulationState::default();
        simulation.start_new_game(2);
        let table_state = TableInputState::default();
        let _ = simulation.add_ball(Vec2::new(100.0, 420.0));

        drain.tick(&mut simulation, &table_state, 0.0);
        assert_eq!(simulation.player_scores[0].ball_count, 3);

        drain.tick(&mut simulation, &table_state, 1.0);

        assert_eq!(simulation.player_scores[0].ball_count, 2);
        assert_eq!(
            simulation.drain_pending_messages(),
            vec![
                TableMessage::from_code(MessageCode::SwitchToNextPlayer),
                TableMessage::from_code(MessageCode::PlungerStartFeedTimer)
            ]
        );
    }

    #[test]
    fn drain_timer_resolution_consumes_extra_ball_and_restarts_feed() {
        let mut drain = DrainMechanic::new(ComponentId(1), "drain");
        let mut simulation = SimulationState::default();
        simulation.player_scores[0].extra_balls = 1;
        let table_state = TableInputState::default();
        let _ = simulation.add_ball(Vec2::new(100.0, 420.0));

        drain.tick(&mut simulation, &table_state, 0.0);
        drain.tick(&mut simulation, &table_state, 1.0);

        assert_eq!(simulation.player_scores[0].extra_balls, 0);
        assert_eq!(simulation.player_scores[0].ball_count, 3);
        assert_eq!(
            simulation.drain_pending_messages(),
            vec![TableMessage::from_code(MessageCode::PlungerStartFeedTimer)]
        );
    }

    #[test]
    fn drain_timer_resolution_requests_game_over_for_last_ball() {
        let mut drain = DrainMechanic::new(ComponentId(1), "drain");
        let mut simulation = SimulationState::default();
        simulation.player_scores[0].ball_count = 1;
        let table_state = TableInputState::default();
        let _ = simulation.add_ball(Vec2::new(100.0, 420.0));

        drain.tick(&mut simulation, &table_state, 0.0);
        drain.tick(&mut simulation, &table_state, 1.0);

        assert_eq!(simulation.player_scores[0].ball_count, 0);
        assert_eq!(
            simulation.drain_pending_messages(),
            vec![TableMessage::from_code(MessageCode::GameOver)]
        );
    }

    #[test]
    fn last_ball_drain_awards_bonus_via_special_add_score() {
        let mut drain = DrainMechanic::new(ComponentId(1), "drain");
        let mut simulation = SimulationState::default();
        simulation.bonus_score = 25_000;
        simulation.score_multiplier = 4;
        simulation.score_added = 50;
        let table_state = TableInputState::default();
        let _ = simulation.add_ball(Vec2::new(100.0, 420.0));

        drain.tick(&mut simulation, &table_state, 0.0);

        assert_eq!(simulation.score(), 25_000);
        assert!(simulation.ball_in_drain);
    }

    #[test]
    fn tilt_locked_drain_does_not_award_bonus_score() {
        let mut drain = DrainMechanic::new(ComponentId(1), "drain");
        let mut simulation = SimulationState::default();
        simulation.bonus_score = 25_000;
        simulation.tilt_locked = true;
        let table_state = TableInputState::default();
        let _ = simulation.add_ball(Vec2::new(100.0, 420.0));

        drain.tick(&mut simulation, &table_state, 0.0);

        assert_eq!(simulation.score(), 0);
        assert!(simulation.ball_in_drain);
    }
}
