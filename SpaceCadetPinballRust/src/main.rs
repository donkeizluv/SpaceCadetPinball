#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;

// Matches the original: internal canvas the game renders to,
// then scaled up to the window.
const CANVAS_W: u32 = 600;
const CANVAS_H: u32 = 416;

// Default window size from winmain.cpp
const WINDOW_W: u32 = 800;
const WINDOW_H: u32 = 556;

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

    // 3. Hardware-accelerated renderer with vsync
    let mut canvas = window
        .into_canvas()
        .accelerated()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;

    // 4. Set logical size so the game always renders at 600x416
    //    and SDL scales it to whatever the window size is.
    //    This replaces the C++ fullscrn.cpp scaling logic.
    canvas
        .set_logical_size(CANVAS_W, CANVAS_H)
        .map_err(|e| e.to_string())?;

    // 5. Simulate asset loading, then show the window
    // TODO: replace with real asset loading
    canvas.window_mut().show();

    // 6. Event loop
    let mut event_pump = sdl.event_pump()?;
    'running: loop {
        // Input
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'running;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::F11),
                    ..
                } => {
                    toggle_fullscreen(&mut canvas);
                }
                _ => {}
            }
        }

        // Clear with the same dark navy background color as the original
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        // TODO: render game here

        canvas.present();
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
