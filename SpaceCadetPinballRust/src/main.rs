#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

#[path = "GameState/mod.rs"]
mod game_state;

use game_state::GameState;
use sdl2::controller::Button as ControllerButton;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mixer::{self, InitFlag};
use sdl2::mouse::MouseButton;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use std::collections::HashMap;
use std::io::Cursor;
use std::thread;
use std::time::{Duration, Instant};

// Matches the original: internal canvas the game renders to,
// then scaled up to the window.
const CANVAS_W: u32 = 600;
const CANVAS_H: u32 = 416;

// Default window size from winmain.cpp
const WINDOW_W: u32 = 800;
const WINDOW_H: u32 = 556;
const TARGET_UPS: u32 = 120;
const TARGET_FPS: u32 = 60;

fn main() -> Result<(), String> {
    // 1. Init SDL (video + audio, matching SDL_INIT_VIDEO | SDL_INIT_AUDIO)
    let sdl = sdl2::init()?;
    let video = sdl.video()?;

    // 2. Create window hidden first, just like the C++ does
    let window = video
        .window("3D Pinball for Windows - Space Cadet", WINDOW_W, WINDOW_H)
        .position_centered()
        .resizable()
        .hidden() // shown after "assets loaded"
        .build()
        .map_err(|e| e.to_string())?;

    // 3. Load game assets while the window is still hidden.
    let mut game_state = GameState::load_default(false)?;
    game_state.set_resolution(0);

    // 4. Hardware-accelerated renderer with vsync
    let mut canvas = window
        .into_canvas()
        .accelerated()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;

    // 5. Set logical size so the game always renders at 600x416
    //    and SDL scales it to whatever the window size is.
    //    This replaces the C++ fullscrn.cpp scaling logic.
    canvas
        .set_logical_size(CANVAS_W, CANVAS_H)
        .map_err(|e| e.to_string())?;

    // 6. SDL_mixer init (non-fatal, same behavior as C++ which can continue without audio).
    let _mixer_context = match mixer::init(InitFlag::MID) {
        Ok(context) => Some(context),
        Err(error) => {
            eprintln!(
                "Could not initialize SDL MIDI, music might not work. SDL Error: {error}"
            );
            None
        }
    };

    let mixer_opened = mixer::open_audio(
        mixer::DEFAULT_FREQUENCY,
        mixer::DEFAULT_FORMAT,
        mixer::DEFAULT_CHANNELS,
        1024,
    )
    .map(|_| {
        mixer::allocate_channels(32);
        true
    })
    .unwrap_or_else(|error| {
        eprintln!("Could not open audio device, continuing without audio. SDL Error: {error}");
        false
    });

    // 7. Load embedded SDL game controller mappings.
    // Uses the same DB revision referenced by EmbeddedData.cpp in the C++ port.
    let game_controller = sdl.game_controller()?;
    let mut db_reader = Cursor::new(include_bytes!("gamecontrollerdb.txt"));
    if let Err(error) = game_controller.load_mappings_from_read(&mut db_reader) {
        eprintln!("Could not load game controller DB. SDL Error: {error}");
    }

    // 8. Assets are loaded and renderer is ready, show the window.
    canvas.window_mut().show();

    // 9. Event/update/render loop
    let mut event_pump = sdl.event_pump()?;
    main_loop(
        &mut event_pump,
        &mut canvas,
        &mut game_state,
        &game_controller,
    )?;

    if mixer_opened {
        mixer::close_audio();
    }

    Ok(())
}

fn toggle_fullscreen(canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
    use sdl2::video::FullscreenType;
    let window = canvas.window_mut();
    let next = match window.fullscreen_state() {
        FullscreenType::Off => FullscreenType::Desktop,
        _ => FullscreenType::Off,
    };
    window.set_fullscreen(next).ok();
}

struct MainLoopState {
    has_focus: bool,
    no_time_loss: bool,
    show_fps: bool,
    update_counter: u32,
    frame_counter: u32,
    fps_prev_time: Instant,
    frame_start: Instant,
    frame_duration: Duration,
    update_to_frame_counter: f64,
    update_to_frame_ratio: f64,
    target_frame_time: Duration,
    idle_wait_ms: u32,
    mouse_down: bool,
    last_mouse_x: i32,
    last_mouse_y: i32,
    opened_controllers: HashMap<u32, sdl2::controller::GameController>,
}

impl MainLoopState {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            has_focus: true,
            no_time_loss: false,
            show_fps: false,
            update_counter: 0,
            frame_counter: 0,
            fps_prev_time: now,
            frame_start: now,
            frame_duration: duration_from_ups(TARGET_UPS),
            update_to_frame_counter: 0.0,
            update_to_frame_ratio: TARGET_UPS as f64 / TARGET_FPS as f64,
            target_frame_time: duration_from_ups(TARGET_UPS),
            idle_wait_ms: duration_from_ups(TARGET_UPS).as_millis() as u32,
            mouse_down: false,
            last_mouse_x: 0,
            last_mouse_y: 0,
            opened_controllers: HashMap::new(),
        }
    }
}

fn duration_from_ups(ups: u32) -> Duration {
    Duration::from_secs_f64(1.0 / ups as f64)
}

fn duration_mul(duration: Duration, scale: u32) -> Duration {
    Duration::from_secs_f64(duration.as_secs_f64() * scale as f64)
}

fn main_loop(
    event_pump: &mut sdl2::EventPump,
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    game_state: &mut GameState,
    game_controller: &sdl2::GameControllerSubsystem,
) -> Result<(), String> {
    let mut state = MainLoopState::new();

    loop {
        if state.show_fps {
            let now = Instant::now();
            let elapsed = now.duration_since(state.fps_prev_time);
            if elapsed >= Duration::from_secs(1) {
                let elapsed_sec = elapsed.as_secs_f64();
                let title = format!(
                    "Updates/sec = {:02.02} Frames/sec = {:02.02}",
                    state.update_counter as f64 / elapsed_sec,
                    state.frame_counter as f64 / elapsed_sec
                );
                canvas.window_mut().set_title(&title).ok();
                state.update_counter = 0;
                state.frame_counter = 0;
                state.fps_prev_time = now;
            }
        }

        if !process_window_messages(event_pump, canvas, game_state, game_controller, &mut state)? {
            break;
        }

        if !state.has_focus {
            continue;
        }

        if !state.no_time_loss {
            update_game(state.frame_duration, game_state);
            state.update_counter = state.update_counter.saturating_add(1);
        }
        state.no_time_loss = false;

        if state.update_to_frame_counter >= state.update_to_frame_ratio {
            render_game(canvas, game_state);
            state.frame_counter = state.frame_counter.saturating_add(1);
            state.update_to_frame_counter -= state.update_to_frame_ratio;
        }

        let update_end = Instant::now();
        let elapsed = update_end.duration_since(state.frame_start);
        if elapsed < state.target_frame_time {
            thread::sleep(state.target_frame_time - elapsed);
        }

        let frame_end = Instant::now();
        state.frame_duration = std::cmp::min(
            frame_end.duration_since(state.frame_start),
            duration_mul(state.target_frame_time, 2),
        );
        state.frame_start = frame_end;
        state.update_to_frame_counter += 1.0;
    }

    Ok(())
}

fn process_window_messages(
    event_pump: &mut sdl2::EventPump,
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    game_state: &mut GameState,
    game_controller: &sdl2::GameControllerSubsystem,
    state: &mut MainLoopState,
) -> Result<bool, String> {
    if state.has_focus {
        state.idle_wait_ms = state.target_frame_time.as_millis() as u32;
        for event in event_pump.poll_iter() {
            if handle_event(event, canvas, game_state, game_controller, state)? {
                return Ok(false);
            }
        }
        return Ok(true);
    }

    state.idle_wait_ms = (state.idle_wait_ms + state.target_frame_time.as_millis() as u32).min(500);
    if let Some(event) = event_pump.wait_event_timeout(state.idle_wait_ms) {
        state.idle_wait_ms = state.target_frame_time.as_millis() as u32;
        if handle_event(event, canvas, game_state, game_controller, state)? {
            return Ok(false);
        }
    }

    Ok(true)
}

fn update_game(frame_duration: Duration, game_state: &mut GameState) {
    let input = game_state.take_input_snapshot();

    game_state.tick_counter = game_state.tick_counter.saturating_add(1);
    game_state.left_flipper_engaged = input.left_flipper;
    game_state.right_flipper_engaged = input.right_flipper;

    let dt = frame_duration.as_secs_f32();
    if input.plunger_pull {
        game_state.plunger_charge = (game_state.plunger_charge + dt * 1.5).min(1.0);
        game_state.launch_impulse = 0.0;
    } else if game_state.plunger_charge > 0.0 {
        // Release behavior: convert accumulated plunger charge into one-shot launch impulse.
        game_state.launch_impulse = game_state.plunger_charge;
        game_state.plunger_charge = 0.0;
    } else {
        game_state.launch_impulse = 0.0;
    }

    if input.impulses.contains("start") {
        game_state.start_pulses = game_state.start_pulses.saturating_add(1);
    }

    if let Some((dx, dy)) = input.nudge {
        game_state.nudge_integrator.0 += dx;
        game_state.nudge_integrator.1 += dy;
    }

    let _ = input.mouse_left;
}

fn render_game(canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, game_state: &GameState) {
    // Clear with the same dark background used by the original port.
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    if let Some(bitmap) = pick_playfield_bitmap(game_state) {
        let texture_creator = canvas.texture_creator();
        if let Ok(mut texture) = texture_creator.create_texture_streaming(
            PixelFormatEnum::RGBA8888,
            bitmap.width as u32,
            bitmap.height as u32,
        ) {
            let _ = texture.with_lock(None, |pixels, pitch| {
                for y in 0..bitmap.height {
                    for x in 0..bitmap.width {
                        let src = y * bitmap.indexed_stride + x;
                        let value = bitmap.indexed_pixels.get(src).copied().unwrap_or(0);
                        let dst = y * pitch + x * 4;
                        pixels[dst] = value;
                        pixels[dst + 1] = value;
                        pixels[dst + 2] = value;
                        pixels[dst + 3] = 255;
                    }
                }
            });

            let dst = Rect::new(
                bitmap.x_position as i32,
                bitmap.y_position as i32,
                bitmap.width as u32,
                bitmap.height as u32,
            );
            let _ = canvas.copy(&texture, None, Some(dst));
        }
    }

    canvas.present();
}

fn pick_playfield_bitmap(game_state: &GameState) -> Option<&game_state::assets::Bitmap8> {
    game_state
        .dat_file
        .groups
        .iter()
        .filter_map(|group| group.get_bitmap(game_state.resolution))
        .filter(|bitmap| {
            bitmap.width <= CANVAS_W as usize
                && bitmap.height <= CANVAS_H as usize
                && bitmap.x_position >= 0
                && bitmap.y_position >= 0
        })
        .max_by_key(|bitmap| bitmap.width * bitmap.height)
}

fn handle_controller_button_down(
    button: ControllerButton,
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    game_state: &mut GameState,
) -> Result<bool, String> {
    match button {
        ControllerButton::Guide => {
            toggle_fullscreen(canvas);
            Ok(false)
        }
        ControllerButton::Y => {
            game_state.reload_assets()?;
            Ok(false)
        }
        ControllerButton::Back => {
            game_state.input_down("back");
            Ok(true)
        }
        ControllerButton::A => {
            game_state.input_down("plunger_pull");
            Ok(false)
        }
        ControllerButton::Start => {
            game_state.input_down("start");
            Ok(false)
        }
        ControllerButton::LeftShoulder => {
            game_state.input_down("left_flipper");
            Ok(false)
        }
        ControllerButton::RightShoulder => {
            game_state.input_down("right_flipper");
            Ok(false)
        }
        ControllerButton::DPadLeft => {
            game_state.input_down("nudge_left");
            game_state.apply_nudge(-0.02, 0.0);
            Ok(false)
        }
        ControllerButton::DPadRight => {
            game_state.input_down("nudge_right");
            game_state.apply_nudge(0.02, 0.0);
            Ok(false)
        }
        ControllerButton::DPadUp => {
            game_state.input_down("nudge_up");
            game_state.apply_nudge(0.0, 0.02);
            Ok(false)
        }
        ControllerButton::DPadDown => {
            game_state.input_down("nudge_down");
            game_state.apply_nudge(0.0, -0.02);
            Ok(false)
        }
        _ => Ok(false),
    }
}

fn handle_controller_button_up(button: ControllerButton, game_state: &mut GameState) {
    match button {
        ControllerButton::A => game_state.input_up("plunger_pull"),
        ControllerButton::Start => game_state.input_up("start"),
        ControllerButton::LeftShoulder => game_state.input_up("left_flipper"),
        ControllerButton::RightShoulder => game_state.input_up("right_flipper"),
        ControllerButton::DPadLeft => game_state.input_up("nudge_left"),
        ControllerButton::DPadRight => game_state.input_up("nudge_right"),
        ControllerButton::DPadUp => game_state.input_up("nudge_up"),
        ControllerButton::DPadDown => game_state.input_up("nudge_down"),
        ControllerButton::Back => game_state.input_up("back"),
        _ => {}
    }
}

fn handle_event(
    event: Event,
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    game_state: &mut GameState,
    game_controller: &sdl2::GameControllerSubsystem,
    state: &mut MainLoopState,
) -> Result<bool, String> {
    match event {
        Event::Quit { .. }
        | Event::KeyDown {
            keycode: Some(Keycode::Escape),
            ..
        } => Ok(true),
        Event::Window {
            win_event: sdl2::event::WindowEvent::FocusGained,
            ..
        }
        | Event::Window {
            win_event: sdl2::event::WindowEvent::Shown,
            ..
        } => {
            state.has_focus = true;
            state.no_time_loss = true;
            Ok(false)
        }
        Event::Window {
            win_event: sdl2::event::WindowEvent::FocusLost,
            ..
        }
        | Event::Window {
            win_event: sdl2::event::WindowEvent::Hidden,
            ..
        } => {
            state.has_focus = false;
            Ok(false)
        }
        Event::KeyDown {
            keycode: Some(Keycode::F11),
            ..
        } => {
            toggle_fullscreen(canvas);
            Ok(false)
        }
        Event::KeyDown {
            keycode: Some(Keycode::F10),
            ..
        } => {
            state.show_fps = !state.show_fps;
            if !state.show_fps {
                canvas
                    .window_mut()
                    .set_title("3D Pinball for Windows - Space Cadet")
                    .ok();
            }
            Ok(false)
        }
        Event::KeyDown {
            keycode: Some(Keycode::F5),
            ..
        } => {
            // Hot reload allows fast iteration while keeping state ownership explicit.
            game_state.reload_assets()?;
            Ok(false)
        }
        Event::KeyDown {
            keycode: Some(Keycode::LShift),
            repeat: false,
            ..
        } => {
            game_state.input_down("left_flipper");
            Ok(false)
        }
        Event::KeyDown {
            keycode: Some(Keycode::RShift),
            repeat: false,
            ..
        } => {
            game_state.input_down("right_flipper");
            Ok(false)
        }
        Event::KeyDown {
            keycode: Some(Keycode::Space),
            repeat: false,
            ..
        } => {
            game_state.input_down("plunger_pull");
            Ok(false)
        }
        Event::KeyUp {
            keycode: Some(Keycode::LShift),
            ..
        } => {
            game_state.input_up("left_flipper");
            Ok(false)
        }
        Event::KeyUp {
            keycode: Some(Keycode::RShift),
            ..
        } => {
            game_state.input_up("right_flipper");
            Ok(false)
        }
        Event::KeyUp {
            keycode: Some(Keycode::Space),
            ..
        } => {
            game_state.input_up("plunger_pull");
            Ok(false)
        }
        Event::MouseButtonDown {
            mouse_btn: MouseButton::Left,
            x,
            y,
            ..
        } => {
            game_state.input_down("mouse_left");
            state.mouse_down = true;
            state.last_mouse_x = x;
            state.last_mouse_y = y;
            canvas.window_mut().set_grab(true);
            Ok(false)
        }
        Event::MouseButtonUp {
            mouse_btn: MouseButton::Left,
            ..
        } => {
            game_state.input_up("mouse_left");
            if state.mouse_down {
                state.mouse_down = false;
                canvas.window_mut().set_grab(false);
            }
            Ok(false)
        }
        Event::MouseMotion { x, y, .. } => {
            if state.mouse_down {
                let (w, h) = canvas.window().size();
                if w > 0 && h > 0 {
                    let dx = (state.last_mouse_x - x) as f32 / w as f32;
                    let dy = (y - state.last_mouse_y) as f32 / h as f32;
                    game_state.apply_nudge(dx, dy);
                }

                state.last_mouse_x = x;
                state.last_mouse_y = y;
            }
            Ok(false)
        }
        Event::JoyDeviceAdded { which, .. } => {
            if game_controller.is_game_controller(which) {
                match game_controller.open(which) {
                    Ok(controller) => {
                        let id = controller.instance_id();
                        state.opened_controllers.insert(id, controller);
                    }
                    Err(error) => {
                        eprintln!("Could not open game controller {which}: {error}");
                    }
                }
            }
            Ok(false)
        }
        Event::JoyDeviceRemoved { which, .. } => {
            state.opened_controllers.remove(&which);
            Ok(false)
        }
        Event::ControllerButtonDown { which, button, .. } => {
            let _ = which;
            handle_controller_button_down(button, canvas, game_state)
        }
        Event::ControllerButtonUp { which, button, .. } => {
            let _ = which;
            handle_controller_button_up(button, game_state);
            Ok(false)
        }
        _ => Ok(false),
    }
}
