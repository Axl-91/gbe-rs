use pixels::{Pixels, SurfaceTexture};
use std::time::Instant;
use std::{sync::Arc, time::Duration};
use winit::event_loop::ControlFlow;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::Window,
};

use crate::Emulator;

const SCREEN_WIDTH: u32 = 160;
const SCREEN_HEIGHT: u32 = 144;

const FRAME_CYCLES: u32 = 70_224;

const GB_LIGHTEST: [u8; 3] = [155, 188, 15];
const GB_LIGHT: [u8; 3] = [139, 172, 15];
const GB_DARK: [u8; 3] = [48, 98, 48];
const GB_DARKEST: [u8; 3] = [15, 56, 15];
const ALPHA_SOLID: u8 = 255;

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
        frame[rgba_index + 3] = ALPHA_SOLID;
    }
}

struct Frontend {
    emulator: Emulator,
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    next_frame: Instant,
}

impl Frontend {
    fn new(emulator: Emulator) -> Self {
        Self {
            emulator,
            window: None,
            pixels: None,
            next_frame: Instant::now(),
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
        let pixels = Pixels::new(SCREEN_WIDTH, SCREEN_HEIGHT, surface_texture).unwrap();

        self.window = Some(window);
        self.pixels = Some(pixels);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();

        if now >= self.next_frame {
            let mut cycles = 0;

            while cycles < FRAME_CYCLES {
                cycles += self.emulator.step() as u32;
            }

            if let Some(window) = &self.window {
                window.request_redraw();
            }

            // Gameboy speed should be 59.73 FPS
            self.next_frame += Duration::from_secs_f64(1.0 / 59.73);
        }

        event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_frame));
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
