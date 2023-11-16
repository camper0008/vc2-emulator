use std::{
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

pub const SCREEN_ENABLED_LOCATION: u32 = 0x2030;
pub const SCREEN_VRAM_ADDRESS_LOCATION: u32 = 0x2034;
pub const SCREEN_WIDTH_LOCATION: u32 = 0x2038;
pub const SCREEN_HEIGHT_LOCATION: u32 = 0x203C;

pub const SCREEN_WIDTH: u32 = 240;
pub const SCREEN_HEIGHT: u32 = 180;
pub const SCREEN_VRAM_ADDRESS: u32 = 0x3000;

use sdl2::{event::Event, keyboard::Keycode, pixels::Color};
use vc2_vm::Vm;

pub fn window<const MEMORY_BYTES: usize, const HALT_MS: u64>(
    vm: Arc<Mutex<Option<Vm<MEMORY_BYTES, HALT_MS>>>>,
) -> JoinHandle<()> {
    thread::spawn(|| {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("vc2", SCREEN_WIDTH * 2, SCREEN_HEIGHT * 2)
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();

        canvas.set_draw_color(Color::RGB(0, 255, 255));
        canvas.clear();
        canvas.present();
        let mut event_pump = sdl_context.event_pump().unwrap();
        let mut i = 0;
        'running: loop {
            i = (i + 1) % 255;
            canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
            canvas.clear();
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    _ => {}
                }
            }
            // The rest of the game loop goes here...

            canvas.present();
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }
    })
}
