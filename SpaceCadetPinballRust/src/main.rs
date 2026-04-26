#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

pub mod assets;
pub mod engine;
pub mod gameplay;
pub mod platform;

use sdl2::pixels::Color;
use std::time::{Duration, Instant};

fn main() -> Result<(), String> {
    let options = platform::options::Options::default();
    let _audio = platform::audio::initialize();
    let mut app = platform::sdl_app::AppShell::new()?;
    let (mut game_state, asset_status_message) = try_load_game_state();
    let mut current_window_title = asset_status_message.clone();
    let asset_load_failed = game_state.is_none();
    let mut table = gameplay::PinballTable::new();
    let mut render_state = engine::render::RenderState::new();
    let mut stepper = engine::time::FixedStepper::new(Duration::from_secs_f64(1.0 / 120.0));
    let mut last_frame_time = Instant::now();

    app.canvas
        .window_mut()
        .set_title(&current_window_title)
        .map_err(|error| error.to_string())?;

    if options.start_fullscreen {
        platform::fullscreen::set_fullscreen(&mut app.canvas, true)?;
    }

    app.show_window();

    'running: loop {
        for event in app.event_pump.poll_iter() {
            match platform::input::translate_event(&event, &options.input_bindings) {
                Some(platform::input::PlatformEvent::ExitRequested) => break 'running,
                Some(platform::input::PlatformEvent::ToggleFullscreen) => {
                    platform::fullscreen::toggle_fullscreen(&mut app.canvas)?;
                }
                Some(platform::input::PlatformEvent::ActionDown(action)) => {
                    if let Some(state) = &mut game_state {
                        state.input_down(action);
                    }
                }
                Some(platform::input::PlatformEvent::ActionUp(action)) => {
                    if let Some(state) = &mut game_state {
                        state.input_up(action);
                    }
                }
                Some(platform::input::PlatformEvent::Nudge { dx, dy }) => {
                    if let Some(state) = &mut game_state {
                        state.apply_nudge(dx, dy);
                    }
                }
                None => {}
            }
        }

        let frame_delta = last_frame_time.elapsed();
        last_frame_time = Instant::now();
        let simulation_steps = stepper.push_frame_time(frame_delta);
        for _ in 0..simulation_steps {
            if let Some(state) = &mut game_state {
                state.advance_table_bridge();
                table.sync_bridge_state(&state.table_bridge);
            }

            table.tick_components(1.0 / 120.0);
            table.step_simulation(1.0 / 120.0);
        }
        app.clear_frame(Color::RGB(0, 0, 0));
        render_state.begin_debug_frame();
        let table_visuals = table.visual_state();

        if asset_load_failed {
            render_state.draw_status_overlay(&mut app.canvas, Color::RGB(220, 40, 40))?;
        }

        render_state.draw_table_visuals(
            &mut app.canvas,
            game_state.as_ref().map(|state| &state.dat_file),
            game_state
                .as_ref()
                .map(|state| state.resolution)
                .unwrap_or(0),
            game_state
                .as_ref()
                .map(|state| state.asset_revision)
                .unwrap_or_default(),
            table_visuals,
        )?;

        let next_window_title = if let Some(debug_summary) = render_state.debug_summary() {
            format!("{asset_status_message} | {debug_summary}")
        } else {
            asset_status_message.clone()
        };
        if next_window_title != current_window_title {
            app.canvas
                .window_mut()
                .set_title(&next_window_title)
                .map_err(|error| error.to_string())?;
            current_window_title = next_window_title;
        }

        let _ = platform::ui::update();
        app.present_frame();
    }

    Ok(())
}

fn try_load_game_state() -> (Option<engine::GameState>, String) {
    match engine::GameState::load_default(false) {
        Ok(state) => (
            Some(state),
            "SpaceCadetPinballRust - DAT loaded (Space Cadet mode)".to_string(),
        ),
        Err(space_cadet_error) => match engine::GameState::load_default(true) {
            Ok(state) => (
                Some(state),
                format!(
                    "SpaceCadetPinballRust - DAT loaded via Full Tilt fallback (Space Cadet error: {space_cadet_error})"
                ),
            ),
            Err(full_tilt_error) => {
                eprintln!(
                    "DAT load failed. Space Cadet mode: {space_cadet_error}. Full Tilt mode: {full_tilt_error}"
                );
                (
                    None,
                    format!(
                        "SpaceCadetPinballRust - DAT load failed: SC={space_cadet_error} | FT={full_tilt_error}"
                    ),
                )
            }
        },
    }
}
