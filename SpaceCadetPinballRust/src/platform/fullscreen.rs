use sdl2::render::Canvas;
use sdl2::video::{FullscreenType, Window};

pub fn toggle_fullscreen(canvas: &mut Canvas<Window>) -> Result<(), String> {
    let enable_desktop = matches!(canvas.window().fullscreen_state(), FullscreenType::Off);
    set_fullscreen(canvas, enable_desktop)
}

pub fn set_fullscreen(canvas: &mut Canvas<Window>, enabled: bool) -> Result<(), String> {
    let mode = if enabled {
        FullscreenType::Desktop
    } else {
        FullscreenType::Off
    };

    canvas
        .window_mut()
        .set_fullscreen(mode)
        .map_err(|error| error.to_string())
}
