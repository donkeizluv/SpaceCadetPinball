use crate::engine::geom::RectI;
use crate::gameplay::components::group_name::*;

use super::PinballTable;

#[derive(Debug, Clone, Copy)]
pub struct BitmapVisualState {
    pub group_name: &'static str,
    pub dest: RectI,
    pub depth_hint: i32,
    pub fallback_shade: u8,
    pub use_native_position: bool,
    pub use_native_size: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SequenceVisualState {
    pub group_name: &'static str,
    pub frame_fraction: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct NumberWidgetVisualState {
    pub widget_group_name: &'static str,
    pub font_group_name: &'static str,
    pub value: u64,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LightVisualState {
    pub group_name: &'static str,
    pub frame_fraction: f32,
}

#[derive(Debug, Clone, Default)]
pub struct TextBoxVisualState {
    pub group_name: &'static str,
    pub text: String,
}

#[derive(Debug, Clone)]
pub enum TableVisual {
    Bitmap(BitmapVisualState),
    Light(LightVisualState),
    NumberWidget(NumberWidgetVisualState),
    Sequence(SequenceVisualState),
    TextBox(TextBoxVisualState),
}

#[derive(Debug, Clone, Copy, Default)]
pub struct HudVisualState {
    pub score_value: u64,
    pub ball_count: u8,
    pub player_number: u8,
}

#[derive(Debug, Clone, Default)]
pub struct TableVisualState {
    pub hud: HudVisualState,
    pub visuals: Vec<TableVisual>,
}

impl PinballTable {
    pub fn visual_state(&self) -> TableVisualState {
        let ball_visual = self.simulation.ball.as_ref().map(|ball| {
            let bounds = ball.bounds();
            let size = bounds.width.max(bounds.height).max(1);
            BitmapVisualState {
                group_name: BALL_GROUP_NAME,
                dest: RectI::new(bounds.x, bounds.y, size, size),
                depth_hint: bounds.y,
                fallback_shade: if ball.is_launched() { 255 } else { 180 },
                use_native_position: false,
                use_native_size: false,
            }
        });
        let score_value = self
            .simulation
            .launch_count
            .saturating_mul(1000)
            .saturating_add(self.simulation.drain_count.saturating_mul(500));
        let ball_count = u8::from(self.simulation.ball.is_some());
        let player_number = 1;
        let score_widget = NumberWidgetVisualState {
            widget_group_name: SCORE_GROUP_NAME,
            font_group_name: FONT_GROUP_NAME,
            value: score_value,
        };
        let ballcount_widget = NumberWidgetVisualState {
            widget_group_name: BALLCOUNT_GROUP_NAME,
            font_group_name: FONT_GROUP_NAME,
            value: u64::from(ball_count),
        };
        let player_widget = NumberWidgetVisualState {
            widget_group_name: PLAYER_NUMBER_GROUP_NAME,
            font_group_name: FONT_GROUP_NAME,
            value: u64::from(player_number),
        };
        let score_progress = (score_value.min(8_000) as f32 / 8_000.0).clamp(0.0, 1.0);
        let launch_progress = (self.simulation.launch_count.min(6) as f32 / 6.0).clamp(0.0, 1.0);
        let drain_progress = (self.simulation.drain_count.min(6) as f32 / 6.0).clamp(0.0, 1.0);
        let lane_ready_progress = if self
            .simulation
            .ball
            .as_ref()
            .is_some_and(|ball| !ball.is_launched())
        {
            1.0
        } else {
            self.simulation.plunger_charge
        };
        let kickback_progress = if self
            .simulation
            .ball
            .as_ref()
            .is_some_and(|ball| !ball.is_launched())
        {
            self.simulation.plunger_charge.max(0.25)
        } else {
            ((launch_progress * 0.7) + (drain_progress * 0.3)).clamp(0.0, 1.0)
        };
        let bumper_progress = ((score_progress * 0.45)
            + (self.simulation.plunger_charge * 0.35)
            + (launch_progress * 0.20))
            .clamp(0.0, 1.0);
        let flag_progress =
            ((launch_progress * 0.55) + (drain_progress * 0.25) + (score_progress * 0.20))
                .clamp(0.0, 1.0);
        let plunger_visual = SequenceVisualState {
            group_name: PLUNGER_GROUP_NAME,
            frame_fraction: self.simulation.plunger_charge,
        };
        let left_flipper_visual = SequenceVisualState {
            group_name: LEFT_FLIPPER_GROUP_NAME,
            frame_fraction: if self.simulation.left_flipper_active {
                1.0
            } else {
                0.0
            },
        };
        let right_flipper_visual = SequenceVisualState {
            group_name: RIGHT_FLIPPER_GROUP_NAME,
            frame_fraction: if self.simulation.right_flipper_active {
                1.0
            } else {
                0.0
            },
        };
        let mut visuals = Vec::with_capacity(240);
        visuals.push(TableVisual::Bitmap(BitmapVisualState {
            group_name: BACKGROUND_GROUP_NAME,
            dest: RectI::new(0, 0, 0, 0),
            depth_hint: i32::MIN,
            fallback_shade: 0,
            use_native_position: true,
            use_native_size: true,
        }));
        visuals.push(TableVisual::Bitmap(BitmapVisualState {
            group_name: TABLE_GROUP_NAME,
            dest: RectI::new(0, 0, 0, 0),
            depth_hint: -10_000,
            fallback_shade: 0,
            use_native_position: false,
            use_native_size: true,
        }));
        visuals.push(TableVisual::Sequence(plunger_visual));
        visuals.push(TableVisual::Sequence(left_flipper_visual));
        visuals.push(TableVisual::Sequence(right_flipper_visual));

        for (index, group_name) in BUMPER_SEQUENCE_GROUPS.into_iter().enumerate() {
            visuals.push(TableVisual::Sequence(SequenceVisualState {
                group_name,
                frame_fraction: (bumper_progress + index as f32 * 0.11).fract(),
            }));
        }

        for (index, group_name) in FLAG_SEQUENCE_GROUPS.into_iter().enumerate() {
            visuals.push(TableVisual::Sequence(SequenceVisualState {
                group_name,
                frame_fraction: (flag_progress + index as f32 * 0.23).fract(),
            }));
        }

        for group_name in GATE_SEQUENCE_GROUPS {
            visuals.push(TableVisual::Sequence(SequenceVisualState {
                group_name,
                frame_fraction: 0.0,
            }));
        }

        for (index, group_name) in KICKBACK_SEQUENCE_GROUPS.into_iter().enumerate() {
            visuals.push(TableVisual::Sequence(SequenceVisualState {
                group_name,
                frame_fraction: (kickback_progress + index as f32 * 0.17).clamp(0.0, 1.0),
            }));
        }

        for group_name in KICKOUT_SEQUENCE_GROUPS {
            visuals.push(TableVisual::Sequence(SequenceVisualState {
                group_name,
                frame_fraction: 0.0,
            }));
        }

        for group_name in SINK_SEQUENCE_GROUPS {
            visuals.push(TableVisual::Sequence(SequenceVisualState {
                group_name,
                frame_fraction: 0.0,
            }));
        }

        for group_name in ONEWAY_SEQUENCE_GROUPS {
            visuals.push(TableVisual::Sequence(SequenceVisualState {
                group_name,
                frame_fraction: 0.0,
            }));
        }

        for group_name in REBOUNDER_SEQUENCE_GROUPS {
            visuals.push(TableVisual::Sequence(SequenceVisualState {
                group_name,
                frame_fraction: 0.0,
            }));
        }

        for group_name in ROLLOVER_SEQUENCE_GROUPS {
            visuals.push(TableVisual::Sequence(SequenceVisualState {
                group_name,
                frame_fraction: 0.0,
            }));
        }

        for group_name in STATIC_TABLE_SEQUENCE_GROUPS {
            visuals.push(TableVisual::Sequence(SequenceVisualState {
                group_name,
                frame_fraction: 0.0,
            }));
        }

        for group_name in LIGHT_GROUP_SEQUENCE_GROUPS {
            visuals.push(TableVisual::Sequence(SequenceVisualState {
                group_name,
                frame_fraction: 0.0,
            }));
        }

        visuals.push(TableVisual::Sequence(SequenceVisualState {
            group_name: FUEL_BARGRAPH_GROUP_NAME,
            frame_fraction: self.simulation.plunger_charge,
        }));
        visuals.push(TableVisual::Sequence(SequenceVisualState {
            group_name: SKILL_SHOT_LIGHTS_GROUP_NAME,
            frame_fraction: lane_ready_progress,
        }));
        visuals.push(TableVisual::Sequence(SequenceVisualState {
            group_name: GOAL_LIGHTS_GROUP_NAME,
            frame_fraction: launch_progress,
        }));
        visuals.push(TableVisual::Sequence(SequenceVisualState {
            group_name: HYPERSPACE_LIGHTS_GROUP_NAME,
            frame_fraction: drain_progress,
        }));
        visuals.push(TableVisual::Sequence(SequenceVisualState {
            group_name: MIDDLE_CIRCLE_GROUP_NAME,
            frame_fraction: score_progress,
        }));
        visuals.push(TableVisual::Sequence(SequenceVisualState {
            group_name: OUTER_CIRCLE_GROUP_NAME,
            frame_fraction: ((launch_progress + drain_progress) * 0.5).clamp(0.0, 1.0),
        }));
        visuals.push(TableVisual::Sequence(SequenceVisualState {
            group_name: WORM_HOLE_LIGHTS_GROUP_NAME,
            frame_fraction: ((score_progress + launch_progress) * 0.5).clamp(0.0, 1.0),
        }));

        for group_name in TARGET_SEQUENCE_GROUPS {
            visuals.push(TableVisual::Sequence(SequenceVisualState {
                group_name,
                frame_fraction: 0.0,
            }));
        }

        for group_name in TRIPWIRE_SEQUENCE_GROUPS {
            visuals.push(TableVisual::Sequence(SequenceVisualState {
                group_name,
                frame_fraction: 0.0,
            }));
        }

        if let Some(ball) = self.simulation.ball.as_ref() {
            visuals.push(TableVisual::Light(LightVisualState {
                group_name: BALL_READY_LIGHT_GROUP_NAME,
                frame_fraction: if ball.is_launched() { 0.0 } else { 1.0 },
            }));
        }

        visuals.push(TableVisual::Light(LightVisualState {
            group_name: LEFT_FLIPPER_LIGHT_GROUP_NAME,
            frame_fraction: if self.simulation.left_flipper_active {
                1.0
            } else {
                0.0
            },
        }));
        visuals.push(TableVisual::Light(LightVisualState {
            group_name: RIGHT_FLIPPER_LIGHT_GROUP_NAME,
            frame_fraction: if self.simulation.right_flipper_active {
                1.0
            } else {
                0.0
            },
        }));

        for group_name in DEFAULT_TABLE_LIGHT_GROUPS {
            visuals.push(TableVisual::Light(LightVisualState {
                group_name,
                frame_fraction: 0.0,
            }));
        }

        for group_name in DEFAULT_ROLLOVER_LIGHT_GROUPS {
            visuals.push(TableVisual::Light(LightVisualState {
                group_name,
                frame_fraction: 0.0,
            }));
        }

        for group_name in DEFAULT_FUEL_ROLLOVER_LIGHT_GROUPS {
            visuals.push(TableVisual::Light(LightVisualState {
                group_name,
                frame_fraction: 0.0,
            }));
        }

        for (group_name, threshold) in PLUNGER_CHARGE_LIGHT_GROUPS {
            visuals.push(TableVisual::Light(LightVisualState {
                group_name,
                frame_fraction: if self.simulation.plunger_charge >= threshold {
                    1.0
                } else {
                    0.0
                },
            }));
        }

        visuals.push(TableVisual::Light(LightVisualState {
            group_name: BALL_IN_PLAY_LIGHT_GROUP_NAME,
            frame_fraction: if self.simulation.ball.is_some() {
                1.0
            } else {
                0.0
            },
        }));
        visuals.push(TableVisual::Light(LightVisualState {
            group_name: BALL_DRAINED_LIGHT_GROUP_NAME,
            frame_fraction: if self.simulation.ball.is_none() && self.simulation.drain_count > 0 {
                1.0
            } else {
                0.0
            },
        }));

        for (group_name, threshold) in SCORE_MILESTONE_LIGHT_GROUPS {
            visuals.push(TableVisual::Light(LightVisualState {
                group_name,
                frame_fraction: if score_value >= threshold { 1.0 } else { 0.0 },
            }));
        }

        for (group_name, threshold) in LAUNCH_MILESTONE_LIGHT_GROUPS {
            visuals.push(TableVisual::Light(LightVisualState {
                group_name,
                frame_fraction: if self.simulation.launch_count >= threshold {
                    1.0
                } else {
                    0.0
                },
            }));
        }

        for (group_name, threshold) in DRAIN_MILESTONE_LIGHT_GROUPS {
            visuals.push(TableVisual::Light(LightVisualState {
                group_name,
                frame_fraction: if self.simulation.drain_count >= threshold {
                    1.0
                } else {
                    0.0
                },
            }));
        }

        visuals.push(TableVisual::NumberWidget(score_widget));
        visuals.push(TableVisual::NumberWidget(ballcount_widget));
        visuals.push(TableVisual::NumberWidget(player_widget));

        if let Some(text) = self.simulation.info_box.current_text().map(str::to_owned) {
            visuals.push(TableVisual::TextBox(TextBoxVisualState {
                group_name: INFO_TEXT_BOX_GROUP_NAME,
                text,
            }));
        }

        if let Some(text) = self
            .simulation
            .mission_box
            .current_text()
            .map(str::to_owned)
        {
            visuals.push(TableVisual::TextBox(TextBoxVisualState {
                group_name: MISSION_TEXT_BOX_GROUP_NAME,
                text,
            }));
        }

        if let Some(ball_visual) = ball_visual {
            visuals.push(TableVisual::Bitmap(ball_visual));
        }

        TableVisualState {
            hud: HudVisualState {
                score_value,
                ball_count,
                player_number,
            },
            visuals,
        }
    }
}
