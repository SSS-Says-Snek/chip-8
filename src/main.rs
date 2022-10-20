mod chip8;
mod utils;

use std::time::{self, Duration, UNIX_EPOCH};

use chip8::Chip8;
use utils::get_input;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

fn render(
    scale: u32,
    display_width: u32,
    display_height: u32,
    canvas: &mut Canvas<Window>,
    buffer: &Vec<u8>,
) {
    // Fill with buffer
    let scl = scale as usize;
    let diswidth = display_width as usize;
    let disheight = display_height as usize;

    for x in 0..diswidth {
        for y in 0..disheight {
            if buffer[y * diswidth + x] > 0 {
                // Foreground
                canvas.set_draw_color((0xAB, 0xAE, 0xCB));
                canvas
                    .fill_rect(Rect::new(
                        (x * scl) as i32,
                        (y * scl) as i32,
                        scl as u32,
                        scl as u32,
                    ))
                    .unwrap();
            } else {
                // Background
                canvas.set_draw_color((0x10, 0x10, 0x20));
                canvas
                    .fill_rect(Rect::new(
                        (x * scl) as i32,
                        (y * scl) as i32,
                        scl as u32,
                        scl as u32,
                    ))
                    .unwrap();
            }
        }
    }

    canvas.present();
}

fn now() -> Duration {
    time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
}

fn main() {
    let (filename, _) = get_input("Enter ch8 filename");
    let mut chip = Chip8::new(filename.trim());

    let video_scale = 8;
    let (texture_width, texture_height) = (64, 32);
    let (window_width, window_height) = (texture_width * video_scale, texture_height * video_scale);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("CHIP-8 Emulator", window_width, window_height)
        .position_centered()
        .build()
        .expect("could not initialize video subsystem");

    let mut canvas = window
        .into_canvas()
        .build()
        .expect("could not make a canvas");

    let mut last_cycle_time = now().as_millis();

    'running: loop {
        for event in sdl_context.event_pump().unwrap().poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                }
                | Event::KeyDown {
                    keycode: Some(Keycode::CapsLock),
                    ..
                } => break 'running,

                Event::KeyDown {
                    keycode: Some(i), ..
                } => match i {
                    Keycode::X => {
                        chip.keys[0] = true;
                    }
                    Keycode::Num1 => chip.keys[1] = true,
                    Keycode::Num2 => chip.keys[2] = true,
                    Keycode::Num3 => chip.keys[3] = true,
                    Keycode::Q => chip.keys[4] = true,
                    Keycode::W => chip.keys[5] = true,
                    Keycode::E => chip.keys[6] = true,
                    Keycode::A => chip.keys[7] = true,
                    Keycode::S => chip.keys[8] = true,
                    Keycode::D => chip.keys[9] = true,
                    Keycode::Z => chip.keys[10] = true,
                    Keycode::C => chip.keys[11] = true,
                    Keycode::Num4 => chip.keys[12] = true,
                    Keycode::R => chip.keys[13] = true,
                    Keycode::F => chip.keys[14] = true,
                    Keycode::V => chip.keys[15] = true,
                    _ => {}
                },
                Event::KeyUp {
                    keycode: Some(i), ..
                } => match i {
                    Keycode::X => chip.keys[0] = false,
                    Keycode::Num1 => chip.keys[1] = false,
                    Keycode::Num2 => chip.keys[2] = false,
                    Keycode::Num3 => chip.keys[3] = false,
                    Keycode::Q => chip.keys[4] = false,
                    Keycode::W => chip.keys[5] = false,
                    Keycode::E => chip.keys[6] = false,
                    Keycode::A => chip.keys[7] = false,
                    Keycode::S => chip.keys[8] = false,
                    Keycode::D => chip.keys[9] = false,
                    Keycode::Z => chip.keys[10] = false,
                    Keycode::C => chip.keys[11] = false,
                    Keycode::Num4 => chip.keys[12] = false,
                    Keycode::R => chip.keys[13] = false,
                    Keycode::F => chip.keys[14] = false,
                    Keycode::V => chip.keys[15] = false,
                    _ => {}
                },

                _ => {}
            }
        }

        let current_time = now().as_millis();

        if current_time - last_cycle_time > 1 {
            last_cycle_time = current_time;

            chip.cycle();

            render(
                video_scale,
                texture_width,
                texture_height,
                &mut canvas,
                &chip.video,
            );
        }
    }
}
