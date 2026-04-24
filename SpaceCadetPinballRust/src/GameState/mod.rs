pub mod assets;

use std::collections::HashSet;
use std::env;
use std::path::PathBuf;

use assets::DatFile;

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
}

impl GameState {
	pub fn load_default(full_tilt_mode: bool) -> Result<Self, String> {
		let dat_path = locate_dat_path()?;
		Self::load_from_path(dat_path, full_tilt_mode)
	}

	pub fn load_from_path(dat_path: PathBuf, full_tilt_mode: bool) -> Result<Self, String> {
		let dat_file = assets::load_records(&dat_path, full_tilt_mode)?;
		Ok(Self {
			dat_file,
			full_tilt_mode,
			resolution: 0,
			dat_path,
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
		})
	}

	pub fn reload_assets(&mut self) -> Result<(), String> {
		self.dat_file = assets::load_records(&self.dat_path, self.full_tilt_mode)?;
		Ok(())
	}

	pub fn set_resolution(&mut self, resolution: usize) {
		self.resolution = resolution.min(2);
	}

	pub fn group_count(&self) -> usize {
		self.dat_file.groups.len()
	}

	pub fn input_down(&mut self, action: &'static str) {
		if Self::is_impulse_action(action) {
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
		self.pending_impulses.insert("nudge");
	}

	pub fn take_input_snapshot(&mut self) -> InputSnapshot {
		InputSnapshot {
			left_flipper: self.active_actions.contains("left_flipper"),
			right_flipper: self.active_actions.contains("right_flipper"),
			plunger_pull: self.active_actions.contains("plunger_pull"),
			mouse_left: self.active_actions.contains("mouse_left"),
			impulses: std::mem::take(&mut self.pending_impulses),
			nudge: self.pending_nudge.take(),
		}
	}

	fn is_impulse_action(action: &'static str) -> bool {
		matches!(action, "start" | "back" | "nudge_left" | "nudge_right" | "nudge_up" | "nudge_down")
	}
}

fn locate_dat_path() -> Result<PathBuf, String> {
	if let Ok(value) = env::var("PINBALL_DAT") {
		let path = PathBuf::from(value);
		if path.is_file() {
			return Ok(path);
		}
	}

	let cwd = env::current_dir().map_err(|error| format!("failed to get current dir: {error}"))?;
	let candidates = [
		cwd.join("PINBALL.DAT"),
		cwd.join("Assets").join("PINBALL.DAT"),
	];

	for candidate in candidates {
		if candidate.is_file() {
			return Ok(candidate);
		}
	}

	Err("could not find PINBALL.DAT (checked PINBALL_DAT, ./PINBALL.DAT, ./Assets/PINBALL.DAT)"
		.to_string())
}
