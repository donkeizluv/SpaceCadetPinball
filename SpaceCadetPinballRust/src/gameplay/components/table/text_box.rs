use std::collections::VecDeque;

use super::PinballTable;

#[derive(Debug, Clone)]
struct TextBoxMessageState {
    text: String,
    remaining_seconds: Option<f32>,
    low_priority: bool,
}

impl TextBoxMessageState {
    fn new(text: impl Into<String>, duration_seconds: f32, low_priority: bool) -> Self {
        Self {
            text: text.into(),
            remaining_seconds: if duration_seconds < 0.0 {
                None
            } else {
                Some(duration_seconds.max(0.0))
            },
            low_priority,
        }
    }

    fn refresh(&mut self, duration_seconds: f32) {
        self.remaining_seconds = if duration_seconds < 0.0 {
            None
        } else {
            Some(duration_seconds.max(0.0))
        };
    }
}

#[derive(Debug, Clone, Default)]
pub(super) struct TextBoxState {
    current: Option<TextBoxMessageState>,
    queued: VecDeque<TextBoxMessageState>,
}

impl TextBoxState {
    pub(super) fn current_text(&self) -> Option<&str> {
        self.current.as_ref().map(|message| message.text.as_str())
    }

    pub(super) fn display(
        &mut self,
        text: impl Into<String>,
        duration_seconds: f32,
        low_priority: bool,
    ) {
        let text = text.into();
        if text.is_empty() {
            return;
        }

        if let Some(current) = self.current.as_mut()
            && current.text == text
        {
            current.refresh(duration_seconds);
            return;
        }

        if let Some(previous) = self.queued.back_mut()
            && previous.text == text
        {
            previous.refresh(duration_seconds);
            return;
        }

        if self
            .current
            .as_ref()
            .is_some_and(|current| current.remaining_seconds.is_none())
        {
            self.clear(false);
        }

        let message = TextBoxMessageState::new(text, duration_seconds, low_priority);
        if self.current.is_none() {
            self.current = Some(message);
        } else {
            self.queued.push_back(message);
        }
    }

    pub(super) fn clear(&mut self, low_priority_only: bool) {
        if low_priority_only {
            if self
                .current
                .as_ref()
                .is_some_and(|message| message.low_priority)
            {
                self.current = None;
            }
            self.queued.retain(|message| !message.low_priority);
        } else {
            self.current = None;
            self.queued.clear();
        }

        self.promote_next()
    }

    pub(super) fn tick(&mut self, dt: f32) {
        if let Some(current) = self.current.as_mut()
            && let Some(remaining_seconds) = current.remaining_seconds.as_mut()
        {
            *remaining_seconds -= dt.max(0.0);
            if *remaining_seconds <= 0.0 {
                self.current = None;
            }
        }

        self.promote_next();
    }

    fn promote_next(&mut self) {
        if self.current.is_none() {
            self.current = self.queued.pop_front();
        }
    }
}

impl PinballTable {
    pub(super) fn refresh_text_boxes(&mut self) {
        let ball_present = self.simulation.has_active_ball();

        if self.simulation.drain_count > self.simulation.previous_drain_count {
            self.simulation.info_box.display(
                format!(
                    "BALL LOST  PRESS START  DRAINS {}",
                    self.simulation.drain_count
                ),
                2.5,
                false,
            );
        }

        if ball_present && !self.simulation.previous_ball_present {
            self.simulation.info_box.display("BALL IN PLAY", 1.5, false);
        }

        if self.simulation.launch_count > self.simulation.previous_launch_count {
            self.simulation.mission_box.display(
                format!("LAUNCH {} READY", self.simulation.launch_count),
                1.5,
                false,
            );
        }

        self.simulation.info_box.clear(true);
        if let Some(text) = self.info_status_text() {
            self.simulation.info_box.display(text, -1.0, true);
        }

        self.simulation.mission_box.clear(true);
        if let Some(text) = self.mission_status_text() {
            self.simulation.mission_box.display(text, -1.0, true);
        }

        self.simulation.previous_ball_present = ball_present;
        self.simulation.previous_launch_count = self.simulation.launch_count;
        self.simulation.previous_drain_count = self.simulation.drain_count;
    }

    fn info_status_text(&self) -> Option<String> {
        if self.simulation.game_over {
            if self.simulation.game_over_ready_for_restart() {
                Some("PRESS START FOR NEW GAME".to_string())
            } else {
                Some("GAME OVER".to_string())
            }
        } else if !self.simulation.has_active_ball() {
            if self.simulation.drain_count > 0 {
                Some(format!(
                    "BALL LOST  PRESS START  DRAINS {}",
                    self.simulation.drain_count
                ))
            } else {
                Some("PRESS START TO SERVE BALL".to_string())
            }
        } else if self.simulation.plunger_charge > 0.05 {
            Some(format!(
                "PLUNGER CHARGE {}%",
                (self.simulation.plunger_charge * 100.0).round() as i32
            ))
        } else {
            Some("BALL IN PLAY".to_string())
        }
    }

    fn mission_status_text(&self) -> Option<String> {
        if self.simulation.game_over {
            Some(format!(
                "FINAL PLAYER {}  SCORE {}",
                self.simulation.player_number(),
                self.simulation.score()
            ))
        } else {
            Some(format!(
                "PLAYER {}  LAUNCHES {}  SCORE {}",
                self.simulation.player_number(),
                self.simulation.launch_count,
                self.simulation.score()
            ))
        }
    }
}
