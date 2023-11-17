use std::{
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

pub const SCREEN_ENABLED_LOCATION: u32 = 0x2030;
pub const SCREEN_VRAM_ADDRESS_LOCATION: u32 = 0x2034;
pub const SCREEN_WIDTH_LOCATION: u32 = 0x2038;
pub const SCREEN_HEIGHT_LOCATION: u32 = 0x203C;

pub const SCREEN_WIDTH: u32 = 120;
pub const SCREEN_HEIGHT: u32 = 90;
pub const SCREEN_VRAM_ADDRESS: u32 = 0x3000;
pub const SCALE: u32 = 4;

use sdl2::{event::Event, pixels::Color, rect::Rect, render::WindowCanvas};
use vc2_vm::Vm;

use crate::utils::sleep;

fn render_canvas(canvas: &mut WindowCanvas, vm: &Vm) -> Result<(), String> {
    let vram_address = vm.memory_value(&SCREEN_VRAM_ADDRESS_LOCATION)?;
    for x in 0..SCREEN_WIDTH {
        for y in 0..SCREEN_HEIGHT {
            let pixel = vm
                .memory_value(&(vram_address + x + y * SCREEN_WIDTH))
                .unwrap();
            let r = ((pixel & 0xFF000000) >> 24) as u8;
            let g = ((pixel & 0x00FF0000) >> 16) as u8;
            let b = ((pixel & 0x0000FF00) >> 8) as u8;
            canvas.set_draw_color(Color::RGB(r, g, b));
            let x = (SCALE * x) as i32;
            let y = (SCALE * y) as i32;
            canvas.fill_rect(Rect::new(x, y, SCALE, SCALE))?;
        }
    }
    Ok(())
}

pub fn window(vm: Arc<Mutex<Option<Vm>>>) -> JoinHandle<()> {
    thread::spawn(move || {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("vc2", SCREEN_WIDTH * SCALE, SCREEN_HEIGHT * SCALE)
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();
        let mut event_pump = sdl_context.event_pump().unwrap();
        let mut i = 0.0f32;
        loop {
            {
                let vm = vm.lock().unwrap();
                match *vm {
                    Some(ref vm) => render_canvas(&mut canvas, vm).unwrap(),
                    None => {
                        let v = (i.sin() * 127.0 * 0.5 + 127.0 * 0.5) as u8;
                        canvas.set_draw_color(Color::RGB(v, v, v));
                        i += 1.0 / 60.0;
                        canvas.clear();
                    }
                }

                for event in event_pump.poll_iter() {
                    match event {
                        Event::Quit { .. } => ::std::process::exit(1),
                        _ => {}
                    }
                }
            }
            canvas.present();
            sleep();
        }
    })
}
