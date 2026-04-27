use crate::engine::physics::Ball;
use crate::gameplay::components::{
    ComponentId, ComponentState, GameplayComponent, MessageCode, SimulationState, TableInputState,
    TableMessage,
};

pub struct PlungerMechanic {
    state: ComponentState,
    charging: bool,
    ball_feed_timer_remaining: Option<f32>,
    pullback_timer_remaining: Option<f32>,
    released_timer_remaining: Option<f32>,
    pullback_started: bool,
    pending_relaunches: u32,
    max_pullback: f32,
    pullback_increment: f32,
    pullback_delay: f32,
}

impl PlungerMechanic {
    pub fn new(id: ComponentId, name: &'static str) -> Self {
        Self::from_state(ComponentState::new(id, name).with_control("PlungerControl"))
    }

    pub fn from_state(state: ComponentState) -> Self {
        Self {
            state,
            charging: false,
            ball_feed_timer_remaining: None,
            pullback_timer_remaining: None,
            released_timer_remaining: None,
            pullback_started: false,
            pending_relaunches: 0,
            max_pullback: 100.0,
            pullback_increment: 2.0,
            pullback_delay: 0.025,
        }
    }

    fn start_pullback(&mut self, simulation: &mut SimulationState, fast: bool) {
        self.charging = true;
        self.pullback_started = true;
        simulation.plunger_charge = 0.0;
        self.state.sprite_index = 0;
        self.pullback_timer_remaining = Some(if fast {
            self.pullback_delay / 4.0
        } else {
            self.pullback_delay
        });
    }

    fn feed_ball(&mut self, simulation: &mut SimulationState) {
        if simulation.ball.is_some() {
            self.ball_feed_timer_remaining = Some(1.0);
            return;
        }

        simulation.ball = Some(Ball::ready_at(simulation.plunger_position));
        simulation.multiball_count = simulation.multiball_count.saturating_add(1);
        simulation.ball_in_drain = false;
    }

    fn release_pullback(&mut self, simulation: &mut SimulationState) {
        self.charging = false;
        self.pullback_started = false;
        self.pullback_timer_remaining = None;
        self.released_timer_remaining = Some(self.pullback_delay);
        self.state.sprite_index = 0;

        if simulation.plunger_charge > 0.0 {
            if let Some(ball) = simulation.ball.as_mut()
                && !ball.is_launched()
            {
                ball.launch(simulation.plunger_charge);
                simulation.launch_count = simulation.launch_count.saturating_add(1);
            }
        }
    }
}

impl GameplayComponent for PlungerMechanic {
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
        simulation: &mut SimulationState,
        _table_state: &TableInputState,
    ) {
        match message {
            TableMessage::StartGame
            | TableMessage::Code(MessageCode::StartGamePlayer1, _)
            | TableMessage::Code(MessageCode::NewGame, _)
            | TableMessage::Code(MessageCode::PlungerFeedBall, _) => {
                self.feed_ball(simulation);
            }
            TableMessage::Code(MessageCode::PlungerStartFeedTimer, _) => {
                self.ball_feed_timer_remaining = Some(0.96);
            }
            TableMessage::Code(MessageCode::PlungerLaunchBall, _) => {
                self.pullback_started = true;
                simulation.plunger_charge = 1.0;
                self.release_pullback(simulation);
            }
            TableMessage::Code(MessageCode::PlungerRelaunchBall, value) => {
                self.pending_relaunches = self.pending_relaunches.saturating_add(1);
                self.ball_feed_timer_remaining = Some(value.max(0.0));
                self.start_pullback(simulation, true);
            }
            TableMessage::PlungerPressed
            | TableMessage::Code(MessageCode::PlungerInputPressed, _) => {
                if !self.pullback_started {
                    self.start_pullback(simulation, self.pending_relaunches > 0);
                }
            }
            TableMessage::PlungerReleased
            | TableMessage::Code(MessageCode::PlungerInputReleased, _)
            | TableMessage::Code(MessageCode::Resume, _)
            | TableMessage::Code(MessageCode::LooseFocus, _) => {
                if self.pullback_started && self.pending_relaunches == 0 {
                    self.release_pullback(simulation);
                }
            }
            TableMessage::Code(MessageCode::PlayerChanged, _) => {
                self.charging = false;
                self.pullback_started = false;
                self.ball_feed_timer_remaining = None;
                self.pullback_timer_remaining = None;
                self.released_timer_remaining = None;
                self.pending_relaunches = 0;
                simulation.plunger_charge = 0.0;
                self.state.sprite_index = 0;
            }
            TableMessage::Code(MessageCode::SetTiltLock, _) => {
                self.pending_relaunches = 0;
                self.ball_feed_timer_remaining = None;
                simulation.tilt_locked = true;
            }
            TableMessage::Code(MessageCode::Reset, _) => {
                self.charging = false;
                self.pullback_started = false;
                self.ball_feed_timer_remaining = None;
                self.pullback_timer_remaining = None;
                self.released_timer_remaining = None;
                self.pending_relaunches = 0;
                simulation.plunger_charge = 0.0;
                self.state.sprite_index = 0;
            }
            _ => {}
        }
    }

    fn tick(&mut self, simulation: &mut SimulationState, _table_state: &TableInputState, dt: f32) {
        let dt = dt.max(0.0);

        if let Some(timer) = self.ball_feed_timer_remaining.as_mut() {
            *timer -= dt;
            if *timer <= 0.0 {
                self.ball_feed_timer_remaining = None;
                self.feed_ball(simulation);
            }
        }

        if let Some(timer) = self.pullback_timer_remaining.as_mut() {
            *timer -= dt;
            if *timer <= 0.0 {
                simulation.plunger_charge =
                    (simulation.plunger_charge + self.pullback_increment / self.max_pullback)
                        .min(1.0);
                let max_sprite = 7;
                self.state.sprite_index =
                    ((simulation.plunger_charge * max_sprite as f32).floor() as i32)
                        .clamp(0, max_sprite);
                if simulation.plunger_charge < 1.0 {
                    self.pullback_timer_remaining = Some(if self.pending_relaunches > 0 {
                        self.pullback_delay / 4.0
                    } else {
                        self.pullback_delay
                    });
                } else {
                    self.pullback_timer_remaining = None;
                }
            }
        }

        if let Some(timer) = self.released_timer_remaining.as_mut() {
            *timer -= dt;
            if *timer <= 0.0 {
                self.released_timer_remaining = None;
                simulation.plunger_charge = 0.0;
            }
        }

        if self.pending_relaunches > 0 && simulation.ball.as_ref().is_some_and(|ball| ball.is_launched()) {
            self.pending_relaunches -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::gameplay::components::TableMessage;

    use super::*;

    #[test]
    fn plunger_launch_ball_uses_message_path_and_resets_charge() {
        let mut plunger = PlungerMechanic::new(ComponentId(1), "plunger");
        let mut simulation = SimulationState::default();
        let table_state = TableInputState::default();
        simulation.ball = Some(Ball::ready_at(simulation.plunger_position));

        plunger.on_message(
            TableMessage::from_code(MessageCode::PlungerLaunchBall),
            &mut simulation,
            &table_state,
        );
        assert_eq!(simulation.launch_count, 1);
        assert!(simulation.ball.as_ref().is_some_and(Ball::is_launched));

        plunger.tick(&mut simulation, &table_state, 0.025);
        assert_eq!(simulation.plunger_charge, 0.0);
    }

    #[test]
    fn plunger_feed_timer_adds_ready_ball() {
        let mut plunger = PlungerMechanic::new(ComponentId(1), "plunger");
        let mut simulation = SimulationState::default();
        let table_state = TableInputState::default();

        plunger.on_message(
            TableMessage::from_code(MessageCode::PlungerStartFeedTimer),
            &mut simulation,
            &table_state,
        );
        plunger.tick(&mut simulation, &table_state, 0.96);

        assert!(simulation.ball.is_some());
        assert_eq!(simulation.multiball_count, 1);
        assert!(!simulation.ball_in_drain);
    }
}
