use std::collections::HashSet;
use std::path::PathBuf;

use crate::assets::{self, DatFile};
use crate::platform::input;

#[derive(Debug, Clone, Copy, Default)]
pub struct TableBridgeState {
    pub left_flipper: bool,
    pub right_flipper: bool,
    pub plunger_pulling: bool,
    pub last_release_impulse: f32,
    pub pending_start: bool,
    pub pending_nudge: Option<(f32, f32)>,
    pub input_ticks: u64,
}

#[derive(Debug)]
pub struct InputSnapshot {
    pub left_flipper: bool,
    pub right_flipper: bool,
    pub plunger_pull: bool,
    pub mouse_left: bool,
    pub impulses: HashSet<&'static str>,
    pub nudge: Option<(f32, f32)>,
}

#[derive(Debug)]
pub struct GameState {
    pub dat_file: DatFile,
    pub full_tilt_mode: bool,
    pub resolution: usize,
    pub dat_path: PathBuf,
    pub asset_revision: u64,
    pub active_actions: HashSet<&'static str>,
    pending_impulses: HashSet<&'static str>,
    pending_nudge: Option<(f32, f32)>,
    pub tick_counter: u64,
    pub left_flipper_engaged: bool,
    pub right_flipper_engaged: bool,
    pub plunger_charge: f32,
    pub launch_impulse: f32,
    pub start_pulses: u64,
    pub nudge_integrator: (f32, f32),
    pub table_bridge: TableBridgeState,
}

impl GameState {
    pub fn load_default(full_tilt_mode: bool) -> Result<Self, String> {
        let dat_path = assets::embedded::locate_dat_path()?;
        Self::load_from_path(dat_path, full_tilt_mode)
    }

    pub fn load_from_path(dat_path: PathBuf, full_tilt_mode: bool) -> Result<Self, String> {
        let dat_file = assets::load_records(&dat_path, full_tilt_mode)?;
        Ok(Self {
            dat_file,
            full_tilt_mode,
            resolution: 0,
            dat_path,
            asset_revision: 0,
            active_actions: HashSet::new(),
            pending_impulses: HashSet::new(),
            pending_nudge: None,
            tick_counter: 0,
            left_flipper_engaged: false,
            right_flipper_engaged: false,
            plunger_charge: 0.0,
            launch_impulse: 0.0,
            start_pulses: 0,
            nudge_integrator: (0.0, 0.0),
            table_bridge: TableBridgeState::default(),
        })
    }

    pub fn reload_assets(&mut self) -> Result<(), String> {
        self.dat_file = assets::load_records(&self.dat_path, self.full_tilt_mode)?;
        self.asset_revision = self.asset_revision.saturating_add(1);
        Ok(())
    }

    pub fn set_resolution(&mut self, resolution: usize) {
        self.resolution = resolution.min(2);
    }

    pub fn group_count(&self) -> usize {
        self.dat_file.groups.len()
    }

    pub fn input_down(&mut self, action: &'static str) {
        if input::is_impulse_action(action) {
            self.pending_impulses.insert(action);
        } else {
            self.active_actions.insert(action);
        }
    }

    pub fn input_up(&mut self, action: &'static str) {
        self.active_actions.remove(action);
    }

    pub fn apply_nudge(&mut self, dx: f32, dy: f32) {
        self.pending_nudge = Some((dx, dy));
        self.pending_impulses.insert(input::ACTION_NUDGE);
    }

    pub fn take_input_snapshot(&mut self) -> InputSnapshot {
        InputSnapshot {
            left_flipper: self.active_actions.contains(input::ACTION_LEFT_FLIPPER),
            right_flipper: self.active_actions.contains(input::ACTION_RIGHT_FLIPPER),
            plunger_pull: self.active_actions.contains(input::ACTION_PLUNGER_PULL),
            mouse_left: self.active_actions.contains(input::ACTION_MOUSE_LEFT),
            impulses: std::mem::take(&mut self.pending_impulses),
            nudge: self.pending_nudge.take(),
        }
    }

    pub fn advance_table_bridge(&mut self) {
        let snapshot = self.take_input_snapshot();

        self.table_begin_tick();
        self.tick_counter = self.tick_counter.saturating_add(1);

        self.left_flipper_engaged = snapshot.left_flipper;
        self.right_flipper_engaged = snapshot.right_flipper;
        self.table_set_left_flipper(snapshot.left_flipper);
        self.table_set_right_flipper(snapshot.right_flipper);

        if snapshot.plunger_pull {
            self.plunger_charge = (self.plunger_charge + 0.04).min(1.0);
            self.table_set_plunger_pull(true);
        } else {
            if self.table_bridge.plunger_pulling && self.plunger_charge > 0.0 {
                self.launch_impulse = self.plunger_charge;
                self.table_release_plunger(self.launch_impulse);
            }

            self.plunger_charge = 0.0;
            self.table_set_plunger_pull(false);
        }

        if snapshot.impulses.contains(input::ACTION_START) {
            self.start_pulses = self.start_pulses.saturating_add(1);
            self.table_trigger_start();
        }

        if let Some((dx, dy)) = snapshot.nudge {
            self.nudge_integrator.0 += dx;
            self.nudge_integrator.1 += dy;
            self.table_apply_nudge(dx, dy);
        }

        self.table_finalize_tick();
    }

    pub fn table_set_left_flipper(&mut self, active: bool) {
        self.table_bridge.left_flipper = active;
    }

    pub fn table_set_right_flipper(&mut self, active: bool) {
        self.table_bridge.right_flipper = active;
    }

    pub fn table_set_plunger_pull(&mut self, active: bool) {
        self.table_bridge.plunger_pulling = active;
    }

    pub fn table_release_plunger(&mut self, impulse: f32) {
        self.table_bridge.last_release_impulse = impulse;
    }

    pub fn table_trigger_start(&mut self) {
        self.table_bridge.pending_start = true;
    }

    pub fn table_apply_nudge(&mut self, dx: f32, dy: f32) {
        self.table_bridge.pending_nudge = Some((dx, dy));
    }

    pub fn table_begin_tick(&mut self) {
        self.table_bridge.last_release_impulse = 0.0;
        self.table_bridge.pending_start = false;
        self.table_bridge.pending_nudge = None;
    }

    pub fn table_finalize_tick(&mut self) {
        self.table_bridge.input_ticks = self.table_bridge.input_ticks.saturating_add(1);
    }
}
