use pixels::{Pixels, SurfaceTexture};
use std::time::Instant;
use std::{sync::Arc, time::Duration};
use winit::event::{ElementState, KeyEvent};
use winit::event_loop::ControlFlow;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::platform::wayland::WindowAttributesExtWayland;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::Window,
};

use crate::Emulator;
use crate::joypad::Button;

const SCREEN_WIDTH: u32 = 160;
const SCREEN_HEIGHT: u32 = 144;

const WINDOW_SCALE: u32 = 6;
const WINDOW_SCALE_BASE: u32 = 5;

const WINDOW_WIDTH: u32 = 627 * WINDOW_SCALE / WINDOW_SCALE_BASE;
const WINDOW_HEIGHT: u32 = 547 * WINDOW_SCALE / WINDOW_SCALE_BASE;

const LCD_X: u32 = 146 * WINDOW_SCALE / WINDOW_SCALE_BASE;
const LCD_Y: u32 = 132 * WINDOW_SCALE / WINDOW_SCALE_BASE;
const LCD_WIDTH: u32 = 339 * WINDOW_SCALE / WINDOW_SCALE_BASE;
const LCD_HEIGHT: u32 = 311 * WINDOW_SCALE / WINDOW_SCALE_BASE;

const FRAME_CYCLES: u32 = 70_224;
const EMULATION_CHUNK_CYCLES: u32 = 256;

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

fn draw_skin(skin: &image::RgbaImage, pixels: &mut Pixels<'static>) {
    let resized_skin = image::imageops::resize(
        skin,
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
        image::imageops::FilterType::Lanczos3,
    );

    pixels.frame_mut().copy_from_slice(resized_skin.as_raw());
}

fn update_framebuffer(emulator: &Emulator, pixels: &mut Pixels<'static>) {
    let framebuffer = emulator.get_framebuffer();
    let frame = pixels.frame_mut();

    for y in 0..LCD_HEIGHT {
        for x in 0..LCD_WIDTH {
            let source_x = x * SCREEN_WIDTH / LCD_WIDTH;
            let source_y = y * SCREEN_HEIGHT / LCD_HEIGHT;
            let source_index = (source_y * SCREEN_WIDTH + source_x) as usize;

            let [red, green, blue] = gameboy_color(framebuffer[source_index]);
            let destination_x = LCD_X + x;
            let destination_y = LCD_Y + y;
            let destination_index = ((destination_y * WINDOW_WIDTH + destination_x) * 4) as usize;

            frame[destination_index] = red;
            frame[destination_index + 1] = green;
            frame[destination_index + 2] = blue;
            frame[destination_index + 3] = ALPHA_SOLID;
        }
    }
}

fn key_to_button(key: KeyCode) -> Option<Button> {
    match key {
        KeyCode::KeyW => Some(Button::Up),
        KeyCode::KeyA => Some(Button::Left),
        KeyCode::KeyS => Some(Button::Down),
        KeyCode::KeyD => Some(Button::Right),
        KeyCode::KeyJ => Some(Button::A),
        KeyCode::KeyK => Some(Button::B),
        KeyCode::Enter => Some(Button::Start),
        KeyCode::ShiftLeft => Some(Button::Select),
        _ => None,
    }
}

struct Frontend {
    emulator: Emulator,
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    frame_cycles: u32,
    next_frame: Instant,
}

impl Frontend {
    fn new(emulator: Emulator) -> Self {
        Self {
            emulator,
            window: None,
            pixels: None,
            frame_cycles: 0,
            next_frame: Instant::now(),
        }
    }
}

impl ApplicationHandler for Frontend {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("GBE-RS")
                        .with_name("gbe-rs", "gbe-rs")
                        .with_inner_size(winit::dpi::LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT)),
                )
                .unwrap(),
        );

        let size = window.inner_size();
        let surface_texture = SurfaceTexture::new(size.width, size.height, window.clone());
        let mut pixels = Pixels::new(WINDOW_WIDTH, WINDOW_HEIGHT, surface_texture).unwrap();

        let skin = image::open("assets/gameboy_layout.jpg").unwrap().to_rgba8();
        draw_skin(&skin, &mut pixels);

        self.window = Some(window);
        self.pixels = Some(pixels);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();

        if now >= self.next_frame {
            let mut cycles = 0;

            while cycles < EMULATION_CHUNK_CYCLES {
                cycles += self.emulator.step() as u32;
            }

            self.frame_cycles += cycles;

            if self.frame_cycles >= FRAME_CYCLES {
                self.frame_cycles -= FRAME_CYCLES;

                if let Some(window) = &self.window {
                    window.request_redraw();
                }

                // Game Boy speed should be 59.73 FPS.
                self.next_frame += Duration::from_secs_f64(1.0 / 59.73);
            }

            event_loop.set_control_flow(ControlFlow::Poll);
        } else {
            event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_frame));
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
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key),
                        state,
                        repeat,
                        ..
                    },
                ..
            } => {
                if let Some(button) = key_to_button(key) {
                    match state {
                        ElementState::Pressed if !repeat => self.emulator.press_button(button),
                        ElementState::Released => self.emulator.release_button(button),
                        _ => {}
                    }
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
