use pixels::{Pixels, SurfaceTexture};
use std::time::Instant;
use std::{sync::Arc, time::Duration};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::Window,
};

use crate::Emulator;

const GB_LIGHTEST: [u8; 3] = [224, 248, 208];
const GB_LIGHT: [u8; 3] = [136, 192, 112];
const GB_DARK: [u8; 3] = [52, 104, 86];
const GB_DARKEST: [u8; 3] = [8, 24, 32];

fn gameboy_color(pixel: u8) -> [u8; 3] {
    match pixel {
        0 => GB_LIGHTEST,
        1 => GB_LIGHT,
        2 => GB_DARK,
        3 => GB_DARKEST,
        _ => GB_DARKEST,
    }
}

fn update_framebuffer(emulator: &Emulator, pixels: &mut Pixels<'static>) {
    let framebuffer = emulator.get_framebuffer();
    let frame = pixels.frame_mut();

    for (index, pixel) in framebuffer.iter().enumerate() {
        let [red, green, blue] = gameboy_color(*pixel);
        let rgba_index = index * 4;

        frame[rgba_index] = red;
        frame[rgba_index + 1] = green;
        frame[rgba_index + 2] = blue;
        frame[rgba_index + 3] = 255;
    }
}

struct Frontend {
    emulator: Emulator,
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
}

impl Frontend {
    fn new(emulator: Emulator) -> Self {
        Self {
            emulator,
            window: None,
            pixels: None,
        }
    }
}

impl ApplicationHandler for Frontend {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        let size = window.inner_size();

        let surface_texture = SurfaceTexture::new(size.width, size.height, window.clone());

        let pixels = Pixels::new(160, 144, surface_texture).unwrap();

        self.window = Some(window);
        self.pixels = Some(pixels);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let frame_start = Instant::now();
        let mut cycles = 0;

        while cycles < 70_224 {
            cycles += self.emulator.step() as u32;
        }

        let frame_duration = frame_start.elapsed();
        let target_duration = Duration::from_secs_f64(1.0 / 59.73);

        if frame_duration < target_duration {
            std::thread::sleep(target_duration - frame_duration);
        }

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Some(pixels) = &mut self.pixels {
                    update_framebuffer(&self.emulator, pixels);
                    pixels.render().unwrap();
                }
            }
            _ => {}
        }
    }
}

pub fn run(emulator: Emulator) {
    let event_loop = EventLoop::new().unwrap();
    let mut frontend = Frontend::new(emulator);

    event_loop.run_app(&mut frontend).unwrap();
}
