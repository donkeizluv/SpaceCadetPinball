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

fn push_sequence_family(
    visuals: &mut Vec<TableVisual>,
    group_names: &[&'static str],
    base_progress: f32,
    phase_step: f32,
) {
    for (index, group_name) in group_names.iter().copied().enumerate() {
        visuals.push(TableVisual::Sequence(SequenceVisualState {
            group_name,
            frame_fraction: (base_progress + index as f32 * phase_step).fract(),
        }));
    }
}

fn push_light_family(
    visuals: &mut Vec<TableVisual>,
    group_names: &[&'static str],
    base_progress: f32,
    phase_step: f32,
) {
    for (index, group_name) in group_names.iter().copied().enumerate() {
        visuals.push(TableVisual::Light(LightVisualState {
            group_name,
            frame_fraction: (base_progress + index as f32 * phase_step).fract(),
        }));
    }
}

#[derive(Debug, Clone, Copy)]
struct ProgressSignals {
    score: f32,
    launch: f32,
    drain: f32,
    plunger: f32,
    lane_ready: f32,
    target_activity: f32,
    orbit_activity: f32,
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
    ramp: f32,
}

fn light_group_sequence_progress(group_name: &'static str, signals: ProgressSignals) -> f32 {
    match group_name {
        "lchute_tgt_lights" => {
            ((signals.left * 0.45) + (signals.top * 0.20) + (signals.lane_ready * 0.35))
                .clamp(0.0, 1.0)
        }
        "l_trek_lights" => {
            ((signals.left * 0.40) + (signals.top * 0.30) + (signals.launch * 0.30))
                .clamp(0.0, 1.0)
        }
        "right_target_lights" => {
            ((signals.right * 0.30)
                + (signals.top * 0.15)
                + (signals.launch * 0.15)
                + (signals.target_activity * 0.40))
                .clamp(0.0, 1.0)
        }
        "r_trek_lights" => {
            ((signals.right * 0.25)
                + (signals.top * 0.20)
                + (signals.score * 0.15)
                + (signals.orbit_activity * 0.40))
                .clamp(0.0, 1.0)
        }
        "bmpr_inc_lights" | "bumper_target_lights" => {
            ((signals.top * 0.45) + (signals.score * 0.35) + (signals.plunger * 0.20))
                .clamp(0.0, 1.0)
        }
        "bpr_solotgt_lights" => {
            ((signals.right * 0.20)
                + (signals.score * 0.20)
                + (signals.launch * 0.15)
                + (signals.target_activity * 0.45))
                .clamp(0.0, 1.0)
        }
        "bsink_arrow_lights" => {
            ((signals.bottom * 0.45) + (signals.drain * 0.35) + (signals.left * 0.20))
                .clamp(0.0, 1.0)
        }
        "ramp_bmpr_inc_lights" | "ramp_tgt_lights" => signals.ramp,
        "top_circle_tgt_lights" | "top_target_lights" => {
            ((signals.top * 0.20)
                + (signals.score * 0.15)
                + (signals.launch * 0.15)
                + (signals.target_activity * 0.50))
                .clamp(0.0, 1.0)
        }
        _ => ((signals.score * 0.35)
            + (signals.launch * 0.30)
            + (signals.drain * 0.20)
            + (signals.plunger * 0.15))
            .clamp(0.0, 1.0),
    }
}

fn static_table_sequence_progress(group_name: &'static str, signals: ProgressSignals) -> f32 {
    match group_name {
        "ramp" | "ramp_hole" => signals.ramp,
        "v_bloc1" => ((signals.bottom * 0.40)
            + (signals.left * 0.25)
            + (signals.drain * 0.20)
            + (signals.plunger * 0.15))
            .clamp(0.0, 1.0),
        _ => ((signals.lane_ready * 0.30)
            + (signals.launch * 0.35)
            + (signals.score * 0.20)
            + (signals.drain * 0.15))
            .clamp(0.0, 1.0),
    }
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
        let score_progress = self.simulation.visual_signals.score_progress;
        let launch_progress = self.simulation.visual_signals.launch_progress;
        let drain_progress = self.simulation.visual_signals.drain_progress;
        let lane_ready_progress = self.simulation.regions.lane_ready;
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
        let bumper_progress = self.simulation.activities.bumper_activity.max(
            self.simulation.visual_signals.impact_focus * 0.6,
        );
        let kickout_progress = self.simulation.visual_signals.recovery_focus;
        let sink_progress = self.simulation.activities.lower_hazard_activity.max(
            self.simulation.visual_signals.hazard_focus * 0.6,
        );
        let gate_progress = self.simulation.visual_signals.lane_focus;
        let flag_progress =
            ((launch_progress * 0.55) + (drain_progress * 0.25) + (score_progress * 0.20))
                .clamp(0.0, 1.0);
        let oneway_progress = self.simulation.visual_signals.lane_focus;
        let rebounder_progress = self.simulation.visual_signals.impact_focus;
        let rollover_progress = self.simulation.visual_signals.lane_focus;
        let target_progress = self.simulation.activities.target_activity.max(
            self.simulation.visual_signals.target_focus * 0.6,
        );
        let tripwire_progress = self.simulation.activities.orbit_activity.max(
            self.simulation.visual_signals.orbit_focus * 0.6,
        );
        let default_table_light_progress = self.simulation.visual_signals.field_light_focus;
        let rollover_light_progress = self.simulation.visual_signals.rollover_light_focus;
        let fuel_rollover_light_progress = self.simulation.visual_signals.fuel_focus;
        let progress_signals = ProgressSignals {
            score: score_progress,
            launch: launch_progress,
            drain: drain_progress,
            plunger: self.simulation.plunger_charge,
            lane_ready: lane_ready_progress,
            target_activity: self.simulation.activities.target_activity,
            orbit_activity: self.simulation.activities.orbit_activity,
            left: self.simulation.regions.left,
            right: self.simulation.regions.right,
            top: self.simulation.regions.top,
            bottom: self.simulation.regions.bottom,
            ramp: self
                .simulation
                .activities
                .ramp_activity
                .max(self.simulation.regions.ramp * 0.65),
        };
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

        push_sequence_family(&mut visuals, &BUMPER_SEQUENCE_GROUPS, bumper_progress, 0.11);
        push_sequence_family(&mut visuals, &FLAG_SEQUENCE_GROUPS, flag_progress, 0.23);
        push_sequence_family(&mut visuals, &GATE_SEQUENCE_GROUPS, gate_progress, 0.29);
        push_sequence_family(&mut visuals, &KICKBACK_SEQUENCE_GROUPS, kickback_progress, 0.17);
        push_sequence_family(&mut visuals, &KICKOUT_SEQUENCE_GROUPS, kickout_progress, 0.19);
        push_sequence_family(&mut visuals, &SINK_SEQUENCE_GROUPS, sink_progress, 0.13);
        push_sequence_family(&mut visuals, &ONEWAY_SEQUENCE_GROUPS, oneway_progress, 0.07);
        push_sequence_family(&mut visuals, &REBOUNDER_SEQUENCE_GROUPS, rebounder_progress, 0.21);
        push_sequence_family(&mut visuals, &ROLLOVER_SEQUENCE_GROUPS, rollover_progress, 0.05);

        for group_name in STATIC_TABLE_SEQUENCE_GROUPS {
            visuals.push(TableVisual::Sequence(SequenceVisualState {
                group_name,
                frame_fraction: static_table_sequence_progress(group_name, progress_signals),
            }));
        }
        for group_name in LIGHT_GROUP_SEQUENCE_GROUPS {
            visuals.push(TableVisual::Sequence(SequenceVisualState {
                group_name,
                frame_fraction: light_group_sequence_progress(group_name, progress_signals),
            }));
        }

        visuals.push(TableVisual::Sequence(SequenceVisualState {
            group_name: FUEL_BARGRAPH_GROUP_NAME,
            frame_fraction: self.simulation.plunger_charge,
        }));
        visuals.push(TableVisual::Sequence(SequenceVisualState {
            group_name: SKILL_SHOT_LIGHTS_GROUP_NAME,
            frame_fraction: self.simulation.activities.lane_activity.max(lane_ready_progress * 0.6),
        }));
        visuals.push(TableVisual::Sequence(SequenceVisualState {
            group_name: GOAL_LIGHTS_GROUP_NAME,
            frame_fraction: self
                .simulation
                .activities
                .ramp_activity
                .max(launch_progress * 0.65),
        }));
        visuals.push(TableVisual::Sequence(SequenceVisualState {
            group_name: HYPERSPACE_LIGHTS_GROUP_NAME,
            frame_fraction: self
                .simulation
                .activities
                .lower_hazard_activity
                .max(drain_progress * 0.65),
        }));
        visuals.push(TableVisual::Sequence(SequenceVisualState {
            group_name: MIDDLE_CIRCLE_GROUP_NAME,
            frame_fraction: score_progress,
        }));
        visuals.push(TableVisual::Sequence(SequenceVisualState {
            group_name: OUTER_CIRCLE_GROUP_NAME,
            frame_fraction: self
                .simulation
                .activities
                .orbit_activity
                .max(((launch_progress + drain_progress) * 0.35).clamp(0.0, 1.0)),
        }));
        visuals.push(TableVisual::Sequence(SequenceVisualState {
            group_name: WORM_HOLE_LIGHTS_GROUP_NAME,
            frame_fraction: self
                .simulation
                .activities
                .orbit_activity
                .max((score_progress * 0.20 + launch_progress * 0.20 + target_progress * 0.20).clamp(0.0, 1.0)),
        }));

        push_sequence_family(&mut visuals, &TARGET_SEQUENCE_GROUPS, target_progress, 0.03);
        push_sequence_family(&mut visuals, &TRIPWIRE_SEQUENCE_GROUPS, tripwire_progress, 0.17);

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

        push_light_family(
            &mut visuals,
            &DEFAULT_TABLE_LIGHT_GROUPS,
            default_table_light_progress,
            0.013,
        );
        push_light_family(
            &mut visuals,
            &DEFAULT_ROLLOVER_LIGHT_GROUPS,
            rollover_light_progress,
            0.071,
        );
        push_light_family(
            &mut visuals,
            &DEFAULT_FUEL_ROLLOVER_LIGHT_GROUPS,
            fuel_rollover_light_progress,
            0.11,
        );

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
