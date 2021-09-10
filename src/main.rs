use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;

use std::env;
use std::io;
use std::{thread, time};

use crate::chip8::Chip8;
use crate::chip8::GFX_SIZE;
use crate::chip8::KEYPAD_SIZE;
use crate::chip8::MEMORY_SIZE;
use crate::chip8::FONTSET_SIZE;
use crate::chip8::WIDTH;

mod chip8;

const IPS: usize = 600;
const FPS: usize = 60;
const REGISTER_SIZE: usize = 16;
const STACK_SIZE: usize = 16;
const SCREEN_WIDTH: usize = 640;
const SCREEN_HEIGHT: usize = 320;
const HEIGHT: usize = 32;
const KEYMAP: [Keycode; 16] = [
    Keycode::Num1,
    Keycode::Num2,
    Keycode::Num3,
    Keycode::Num4,
    Keycode::Q,
    Keycode::W,
    Keycode::E,
    Keycode::R,
    Keycode::A,
    Keycode::S,
    Keycode::D,
    Keycode::F,
    Keycode::Z,
    Keycode::X,
    Keycode::C,
    Keycode::V,
];

// TODO: do proper error propagation
fn main() -> Result<(), io::Error> {
    if env::args().len() != 2 {
        panic!("usage: ./chip8 <path-to-ROM-file>");
    }

    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];

    let mut chip8 = Chip8 {
        draw_flag: false,
        gfx: [0; GFX_SIZE],
        key: [false; KEYPAD_SIZE],
        opcode: 0,
        memory: [0; MEMORY_SIZE],
        v: [0; REGISTER_SIZE], // CPU registers
        index: 0,              // Index register / memory address register
        pc: 0x200,             // program counter
        delay_timer: 0,
        sound_timer: 0,
        stack: [0; STACK_SIZE],
        sp: 0, // stack pointer
        fontset: [0; FONTSET_SIZE],
    };

    chip8.init();
    chip8.load_rom(file_path)?;

    // SDL2
    let sdl_context = sdl2::init().unwrap();
    let video_subsys = sdl_context.video().unwrap();
    let window = video_subsys
        .window("CHIP-8", SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::ARGB8888, WIDTH as u32, HEIGHT as u32)
        .unwrap();
    let mut pixel_data: [u8; GFX_SIZE * 4] = [0; GFX_SIZE * 4];
    let mut events = sdl_context.event_pump().unwrap();

    let ipf = IPS / FPS; // instructions per frame

    'main: loop {
        // perform the instructions before ticking the timers
        for _ in 0..ipf {
            chip8.emulate_cycle();
        }
        chip8.update_timers();

        // sleep every frame instead of every instruction
        thread::sleep(time::Duration::from_millis(1000 / 60)); // 16.667 milliseconds should be "almost" accurate

        for event in events.poll_iter() {
            match event {
                Event::Quit { .. } => break 'main,
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    if keycode == Keycode::Escape {
                        break 'main;
                    } else {
                        for i in 0..KEYPAD_SIZE {
                            if keycode == KEYMAP[i] {
                                chip8.key[i] = true;
                            }
                        }
                    }
                }
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => {
                    for i in 0..KEYPAD_SIZE {
                        if keycode == KEYMAP[i] {
                            chip8.key[i] = false;
                        }
                    }
                }
                _ => {}
            }
        }

        if chip8.draw_flag {
            chip8.draw_flag = false;

            for i in 0..GFX_SIZE {
                let offset = i * 4;
                let pixel: u8 = chip8.gfx[i];
                pixel_data[offset] = 0xFF;
                pixel_data[offset + 1] = pixel * 0xFF;
                pixel_data[offset + 2] = pixel * 0xFF;
                pixel_data[offset + 3] = pixel * 0xFF;
            }

            texture
                .update(None, &pixel_data, 4 * WIDTH as usize)
                .unwrap();
            canvas.clear();
            canvas.copy(&texture, None, None).unwrap();
            canvas.present();
        }
    }

    Ok(())
}
