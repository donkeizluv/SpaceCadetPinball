use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::{EventPump, Sdl};

pub const WINDOW_TITLE: &str = "3D Pinball for Windows - Space Cadet";
pub const CANVAS_W: u32 = 600;
pub const CANVAS_H: u32 = 416;
pub const WINDOW_W: u32 = 800;
pub const WINDOW_H: u32 = 556;

pub struct AppShell {
    _sdl: Sdl,
    pub canvas: Canvas<Window>,
    pub event_pump: EventPump,
}

impl AppShell {
    pub fn new() -> Result<Self, String> {
        let sdl = sdl2::init()?;
        let video = sdl.video()?;
        let window = video
            .window(WINDOW_TITLE, WINDOW_W, WINDOW_H)
            .position_centered()
            .resizable()
            .hidden()
            .build()
            .map_err(|error| error.to_string())?;

        let mut canvas = window
            .into_canvas()
            .accelerated()
            .present_vsync()
            .build()
            .map_err(|error| error.to_string())?;
        canvas
            .set_logical_size(CANVAS_W, CANVAS_H)
            .map_err(|error| error.to_string())?;

        let event_pump = sdl.event_pump()?;
        Ok(Self {
            _sdl: sdl,
            canvas,
            event_pump,
        })
    }

    pub fn show_window(&mut self) {
        self.canvas.window_mut().show();
    }

    pub fn clear_frame(&mut self, color: Color) {
        self.canvas.set_draw_color(color);
        self.canvas.clear();
    }

    pub fn present_frame(&mut self) {
        self.canvas.present();
    }
}
