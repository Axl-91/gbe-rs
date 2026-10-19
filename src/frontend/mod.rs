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

mod audio;

use crate::Emulator;
use crate::joypad::Button;

/// Game Boy screen size in pixels.
const SCREEN_WIDTH: u32 = 160;
const SCREEN_HEIGHT: u32 = 144;

const WINDOW_SCALE: u32 = 6;
const WINDOW_SCALE_BASE: u32 = 5;

/// Emulator window screen size in pixels.
const WINDOW_WIDTH: u32 = 627 * WINDOW_SCALE / WINDOW_SCALE_BASE;
const WINDOW_HEIGHT: u32 = 547 * WINDOW_SCALE / WINDOW_SCALE_BASE;

/// Initial position and size where LCD is place on the window
const LCD_X: u32 = 146 * WINDOW_SCALE / WINDOW_SCALE_BASE;
const LCD_Y: u32 = 132 * WINDOW_SCALE / WINDOW_SCALE_BASE;
const LCD_WIDTH: u32 = 339 * WINDOW_SCALE / WINDOW_SCALE_BASE;
const LCD_HEIGHT: u32 = 311 * WINDOW_SCALE / WINDOW_SCALE_BASE;

const FRAME_CYCLES: u32 = 70_224;
const EMULATION_CHUNK_CYCLES: u32 = 256;

const SKIN_BYTES: &[u8] = include_bytes!("../../assets/gameboy_layout.png");

fn draw_skin(pixels: &mut Pixels<'static>) {
    let skin = image::load_from_memory_with_format(SKIN_BYTES, image::ImageFormat::Png)
        .unwrap()
        .to_rgba8();

    pixels.frame_mut().copy_from_slice(skin.as_raw());
}

/// Classic DMG green palette, indexed by the 2-bit shade value emitted by the PPU.
/// Stored as RGBA so a whole pixel can be written with a single copy.
const PALETTE: [[u8; 4]; 4] = [
    [155, 188, 15, 255], // 0: lightest
    [139, 172, 15, 255], // 1: light
    [48, 98, 48, 255],   // 2: dark
    [15, 56, 15, 255],   // 3: darkest
];

/// Precomputed nearest-neighbor lookup tables that map every pixel of the
/// on-screen LCD area to a pixel of the native 160x144 Game Boy framebuffer.
///
/// Building these once avoids doing two multiplications and two divisions
/// per pixel on every frame.
struct LcdMap {
    /// For each destination column (`0..LCD_WIDTH`), the source column.
    src_x: Vec<usize>,
    /// For each destination row (`0..LCD_HEIGHT`), the source row.
    src_y: Vec<usize>,
}

impl LcdMap {
    /// Builds the lookup tables from the current layout constants.
    /// If `WINDOW_SCALE` changes, the tables adapt automatically.
    fn new() -> Self {
        Self {
            src_x: (0..LCD_WIDTH)
                .map(|x| (x * SCREEN_WIDTH / LCD_WIDTH) as usize)
                .collect(),
            src_y: (0..LCD_HEIGHT)
                .map(|y| (y * SCREEN_HEIGHT / LCD_HEIGHT) as usize)
                .collect(),
        }
    }
}

/// Scales the emulator framebuffer into the LCD region of the window buffer.
///
/// Only the LCD rectangle is written; the rest of the buffer (the console
/// skin drawn by `draw_skin`) is left untouched.
///
/// Because the LCD is larger than the source screen, consecutive destination
/// rows often map to the same source row. The first one is rendered pixel by
/// pixel; the repeats are produced with a single `memcpy` (`copy_within`).
fn update_framebuffer(emulator: &Emulator, pixels: &mut Pixels<'static>, map: &LcdMap) {
    let framebuffer = emulator.get_framebuffer();
    let frame = pixels.frame_mut();

    let stride = WINDOW_WIDTH as usize * 4;
    let lcd_row_bytes = LCD_WIDTH as usize * 4;
    let screen_w = SCREEN_WIDTH as usize;

    // Track the last rendered row so identical source rows can be copied.
    let mut prev_src_y = usize::MAX;
    let mut prev_row_start = 0;

    for (y, &src_y) in map.src_y.iter().enumerate() {
        // Byte offset of this destination row inside the window buffer.
        let row_start = (LCD_Y as usize + y) * stride + LCD_X as usize * 4;

        if src_y == prev_src_y {
            // Same source row as the previous destination row:
            // duplicate the already-rendered row instead of recomputing it.
            frame.copy_within(prev_row_start..prev_row_start + lcd_row_bytes, row_start);
        } else {
            let src_row = &framebuffer[src_y * screen_w..(src_y + 1) * screen_w];
            let dst_row = &mut frame[row_start..row_start + lcd_row_bytes];

            // View the row as whole RGBA pixels; the remainder is empty
            // because `dst_row` is exactly `LCD_WIDTH * 4` bytes long.
            let (dst_pixels, _) = dst_row.as_chunks_mut::<4>();

            // `& 3` keeps the palette index in range (0..=3).
            for (dst, &sx) in dst_pixels.iter_mut().zip(&map.src_x) {
                *dst = PALETTE[(src_row[sx] & 3) as usize];
            }
        }

        prev_src_y = src_y;
        prev_row_start = row_start;
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

/// Application state: owns the emulator and the window/rendering resources.
struct Frontend {
    /// The Game Boy core being driven by this frontend.
    emulator: Emulator,
    /// Audio output stream; kept alive for the application's lifetime.
    audio: audio::AudioOutput,
    /// Window handle; created lazily in `resumed`.
    window: Option<Arc<Window>>,
    /// Pixel buffer and GPU surface; created together with the window.
    pixels: Option<Pixels<'static>>,
    /// Precomputed source-to-LCD pixel mapping (see `LcdMap`).
    lcd_map: LcdMap,
    /// Emulated cycles accumulated toward the next frame.
    frame_cycles: u32,
    /// Deadline for emulating the next frame (frame pacing).
    next_frame: Instant,
}

impl Frontend {
    fn new(emulator: Emulator) -> Self {
        let audio = audio::AudioOutput::new()
            .unwrap_or_else(|error| panic!("Failed to initialize audio output: {error}"));

        Self {
            emulator,
            audio,
            window: None,
            pixels: None,
            lcd_map: LcdMap::new(),
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

        draw_skin(&mut pixels);

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

            let samples = self.emulator.take_audio_samples();
            self.audio.push_samples(samples);

            self.frame_cycles += cycles;

            if self.frame_cycles >= FRAME_CYCLES {
                self.frame_cycles -= FRAME_CYCLES;

                if let Some(window) = &self.window {
                    window.request_redraw();
                }

                // Game Boy speed should be 59.73 FPS.
                self.next_frame += Duration::from_secs_f64(1.0 / 59.73);

                // If we fell more than 100 ms behind, resync instead of catching up.
                let now = Instant::now();
                if now > self.next_frame + Duration::from_millis(100) {
                    self.next_frame = now;
                }
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
                    update_framebuffer(&self.emulator, pixels, &self.lcd_map);
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
                if key == KeyCode::Escape && state == ElementState::Pressed {
                    event_loop.exit();
                    return;
                };

                if let Some(button) = key_to_button(key) {
                    match state {
                        ElementState::Pressed if !repeat => self.emulator.press_button(button),
                        ElementState::Released => self.emulator.release_button(button),
                        _ => {}
                    }
                };
            }
            _ => {}
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        if let Err(error) = self.emulator.save() {
            log::error!("Failed to save cartridge RAM: {error}");
        }
    }
}

pub fn run(emulator: Emulator) {
    let event_loop = EventLoop::new().unwrap();
    let mut frontend = Frontend::new(emulator);

    event_loop.run_app(&mut frontend).unwrap();
}
