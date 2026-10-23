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

const WINDOW_SCALE: u32 = 7;
const WINDOW_SCALE_BASE: u32 = 5;

const WINDOW_WIDTH: u32 = 627 * WINDOW_SCALE / WINDOW_SCALE_BASE;
const WINDOW_HEIGHT: u32 = 547 * WINDOW_SCALE / WINDOW_SCALE_BASE;

/// Integer scale of the Game Boy screen (160x144 -> 480x432).
const LCD_SCALE: u32 = 3;
const SCALE: usize = LCD_SCALE as usize;
const SCREEN_W: usize = SCREEN_WIDTH as usize;

const LCD_WIDTH: u32 = SCREEN_WIDTH * LCD_SCALE;
const LCD_HEIGHT: u32 = SCREEN_HEIGHT * LCD_SCALE;

// The LCD is centered on the center of the LCD area of the original
// 627x547 skin: x = 146 + 339 / 2, y = 132 + 311 / 2.
const LCD_X: u32 = (2 * 146 + 339) * WINDOW_SCALE / (2 * WINDOW_SCALE_BASE) - LCD_WIDTH / 2;
const LCD_Y: u32 = (2 * 132 + 311) * WINDOW_SCALE / (2 * WINDOW_SCALE_BASE) - LCD_HEIGHT / 2;

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

/// Darkened version of `PALETTE`, used for the gaps between LCD pixels.
const GRID_PALETTE: [[u8; 4]; 4] = [
    [124, 150, 12, 255], // 0: lightest
    [111, 138, 12, 255], // 1
    [38, 78, 38, 255],   // 2
    [12, 45, 12, 255],   // 3: darkest
];

/// Returns one LCD row as blocks of `SCALE` RGBA pixels, one block per source pixel.
fn lcd_row(frame: &mut [u8], start: usize) -> &mut [[[u8; 4]; SCALE]] {
    let bytes = &mut frame[start..start + LCD_WIDTH as usize * 4];
    let (pixels, _) = bytes.as_chunks_mut::<4>();
    let (blocks, _) = pixels.as_chunks_mut::<SCALE>();
    blocks
}

/// Draws the emulator framebuffer into the LCD area with an LCD-like grid.
///
/// Each Game Boy pixel becomes a SCALE x SCALE block: its top row and left
/// column are drawn with `GRID_PALETTE`, the rest with `PALETTE`. Only the
/// first body row of each block is rendered; the others are copied.
fn update_framebuffer(emulator: &Emulator, pixels: &mut Pixels<'static>) {
    let framebuffer = emulator.get_framebuffer();
    let frame = pixels.frame_mut();
    let stride = WINDOW_WIDTH as usize * 4;
    let lcd_row_bytes = LCD_WIDTH as usize * 4;

    for (src_y, src_row) in framebuffer.as_chunks::<SCREEN_W>().0.iter().enumerate() {
        let grid_row = (LCD_Y as usize + src_y * SCALE) * stride + LCD_X as usize * 4;
        let body_row = grid_row + stride;

        // Top row of the block: the horizontal grid line.
        for (block, &shade) in lcd_row(frame, grid_row).iter_mut().zip(src_row) {
            block.fill(GRID_PALETTE[(shade & 3) as usize]);
        }

        // First body row: the vertical grid line, then the pixel color.
        for (block, &shade) in lcd_row(frame, body_row).iter_mut().zip(src_row) {
            let shade = (shade & 3) as usize;
            block[0] = GRID_PALETTE[shade];
            block[1..].fill(PALETTE[shade]);
        }

        // Remaining body rows are identical: duplicate them.
        for row in 2..SCALE {
            frame.copy_within(body_row..body_row + lcd_row_bytes, grid_row + row * stride);
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
                        .with_title("KaniBoy")
                        .with_name("kaniboy", "kaniboy")
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
