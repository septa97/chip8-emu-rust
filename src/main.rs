use rand::Rng;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::{thread, time};

const IPS: usize = 600;
const FPS: usize = 60;
const MEMORY_SIZE: usize = 4096;
const GFX_SIZE: usize = 2048;
const KEYPAD_SIZE: usize = 16;
const REGISTER_SIZE: usize = 16;
const STACK_SIZE: usize = 16;
const FONTSET_SIZE: usize = 80;
const SCREEN_WIDTH: usize = 640;
const SCREEN_HEIGHT: usize = 320;
const WIDTH: usize = 64;
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

struct Chip8 {
    draw_flag: bool,
    gfx: [u8; GFX_SIZE],
    key: [bool; KEYPAD_SIZE],
    opcode: u16,
    memory: [u8; MEMORY_SIZE],
    v: [u8; 16],  // CPU registers
    index: usize, // Index register / memory address register
    pc: usize,    // program counter
    delay_timer: u8,
    sound_timer: u8,
    stack: [usize; 16],
    sp: usize, // stack pointer
    fontset: [u8; FONTSET_SIZE],
}

impl Chip8 {
    fn update_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                println!("BEEP!")
                // TODO: implement sound
            }

            self.sound_timer -= 1;
        }
    }

    fn init(&mut self) {
        self.fontset = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ];

        for i in 0..FONTSET_SIZE {
            self.memory[i] = self.fontset[i];
        }
    }

    fn load_rom(&mut self, file_path: &String) -> Result<(), io::Error> {
        let mut file = File::open(file_path)?;
        let mut buffer = Vec::new();
        let bytes = file.read_to_end(&mut buffer)?;

        println!("rom size: {} bytes", bytes);
        if MEMORY_SIZE - 0x200 < bytes {
            // TODO: is panic the best practice here? I'm thinking of propagating a custom Error instead
            panic!("ROM too large to fit in memory");
        }

        for i in 0..bytes {
            self.memory[i + 0x200] = buffer[i];
        }

        Ok(())
    }

    fn emulate_cycle(&mut self) {
        // fetch opcode
        let high = (self.memory[self.pc] as u16) << 8;
        let low = self.memory[self.pc + 1] as u16;
        self.opcode = high | low;
        let x = usize::from((self.opcode & 0x0F00) >> 8);
        let y = usize::from((self.opcode & 0x00F0) >> 4);
        let nn = self.opcode as u8 & 0xFF;

        // decode and execute opcode
        match self.opcode & 0xF000 {
            0x0000 => match self.opcode & 0x0FFF {
                0x00E0 => {
                    self.gfx = [0; GFX_SIZE];
                    self.draw_flag = true;
                    self.pc += 2;
                }
                0x00EE => {
                    self.sp -= 1;
                    self.pc = self.stack[self.sp];
                    self.pc += 2;
                }
                // 0NNN (call): Calls RCA 1802 program at address NNN. Not necessary for most ROMs.
                // Only needed if emulating the RCA 1802 processor
                _ => panic!("Unknown opcode!"),
            },
            0x1000 => self.pc = usize::from(self.opcode & 0x0FFF),
            0x2000 => {
                self.stack[self.sp] = self.pc;
                self.sp += 1;
                self.pc = usize::from(self.opcode & 0x0FFF);
            }
            0x3000 => {
                if self.v[x] == nn {
                    self.pc += 2;
                }
                self.pc += 2;
            }
            0x4000 => {
                if self.v[x] != nn {
                    self.pc += 2;
                }
                self.pc += 2;
            }
            0x5000 => {
                if self.v[x] == self.v[y] {
                    self.pc += 2;
                }
                self.pc += 2;
            }
            0x6000 => {
                self.v[x] = nn;
                self.pc += 2;
            }
            0x7000 => {
                self.v[x] = self.v[x].overflowing_add(nn).0;
                self.pc += 2;
            }
            0x8000 => match self.opcode & 0xF00F {
                0x8000 => {
                    self.v[x] = self.v[y];
                    self.pc += 2;
                }
                0x8001 => {
                    self.v[x] |= self.v[y];
                    self.pc += 2;
                }
                0x8002 => {
                    self.v[x] &= self.v[y];
                    self.pc += 2;
                }
                0x8003 => {
                    self.v[x] ^= self.v[y];
                    self.pc += 2;
                }
                0x8004 => {
                    let (result, overflowed) = self.v[x].overflowing_add(self.v[y]);

                    if overflowed {
                        self.v[0xF] = 1;
                    } else {
                        self.v[0xF] = 0;
                    }

                    self.v[x] = result;
                    self.pc += 2;
                }
                0x8005 => {
                    let (result, overflowed) = self.v[x].overflowing_sub(self.v[y]);

                    if overflowed {
                        self.v[0xF] = 0;
                    } else {
                        self.v[0xF] = 1;
                    }

                    self.v[x] = result;
                    self.pc += 2;
                }
                0x8006 => {
                    self.v[0xF] = self.v[x] & 1;
                    self.v[x] >>= 1;

                    self.pc += 2;
                }
                0x8007 => {
                    let (result, overflowed) = self.v[y].overflowing_sub(self.v[x]);

                    if overflowed {
                        self.v[0xF] = 0;
                    } else {
                        self.v[0xF] = 1;
                    }

                    self.v[x] = result;
                    self.pc += 2;
                }
                0x800E => {
                    self.v[0xF] = self.v[x] >> 7;
                    self.v[x] <<= 1;

                    self.pc += 2;
                }
                _ => panic!("Unknown opcode!"),
            },
            0x9000 => {
                if self.v[x] != self.v[y] {
                    self.pc += 2;
                }
                self.pc += 2;
            }
            0xA000 => {
                self.index = usize::from(self.opcode & 0x0FFF);
                self.pc += 2;
            }
            0xB000 => self.pc = usize::from((self.opcode & 0x0FFF) + u16::from(self.v[0])),
            0xC000 => {
                let num: u8 = rand::thread_rng().gen();
                self.v[x] = num & nn;
                self.pc += 2;
            }
            0xD000 => {
                let n = usize::from(self.opcode & 0x000F);

                self.v[0xF] = 0;

                for i in 0..n {
                    let pixel: u8 = self.memory[self.index + i];

                    for j in 0..8 {
                        if (pixel & (0x80 >> j)) != 0 {
                            let idx = (usize::from(self.v[x])
                                + j
                                + ((usize::from(self.v[y]) + i) * WIDTH))
                                % GFX_SIZE;

                            if self.gfx[idx] == 1 {
                                self.v[0xF] = 1;
                            }

                            self.gfx[idx] ^= 1;
                        }
                    }
                }

                self.draw_flag = true;
                self.pc += 2;
            }
            0xE000 => match self.opcode & 0xF0FF {
                0xE09E => {
                    let x = usize::from((self.opcode & 0x0F00) >> 8);
                    let idx = usize::from(self.v[x]);

                    if self.key[idx] {
                        self.pc += 2;
                    }
                    self.pc += 2;
                }
                0xE0A1 => {
                    let idx = usize::from(self.v[x]);

                    if !self.key[idx] {
                        self.pc += 2;
                    }
                    self.pc += 2;
                }
                _ => panic!("Unknown opcode!"),
            },
            0xF000 => match self.opcode & 0xF0FF {
                0xF007 => {
                    self.v[x] = self.delay_timer;
                    self.pc += 2;
                }
                0xF00A => {
                    let mut key_pressed: bool = false;

                    for i in 0..KEYPAD_SIZE {
                        if self.key[i] {
                            self.v[x] = i as u8;
                            key_pressed = true;
                        }
                    }

                    if key_pressed {
                        self.pc += 2;
                    }
                }
                0xF015 => {
                    self.delay_timer = self.v[x];
                    self.pc += 2;
                }
                0xF018 => {
                    self.sound_timer = self.v[x];
                    self.pc += 2;
                }
                0xF01E => {
                    if self.index + self.v[x] as usize > 0xFFF {
                        self.v[0xF] = 1;
                    } else {
                        self.v[0xF] = 0;
                    }

                    self.index += self.v[x] as usize;
                    self.pc += 2;
                }
                0xF029 => {
                    let x = usize::from((self.opcode & 0x0F00) >> 8);

                    self.index = self.v[x] as usize * 5;
                    self.pc += 2;
                }
                0xF033 => {
                    let x = usize::from((self.opcode & 0x0F00) >> 8);

                    self.memory[self.index] = self.v[x] / 100;
                    self.memory[self.index + 1] = (self.v[x] / 10) % 10;
                    self.memory[self.index + 2] = self.v[x] % 10;
                    self.pc += 2;
                }
                0xF055 => {
                    let x = usize::from((self.opcode & 0x0F00) >> 8);

                    for i in 0..x + 1 {
                        self.memory[self.index + i] = self.v[i];
                    }

                    self.pc += 2;
                }
                0xF065 => {
                    let x = usize::from((self.opcode & 0x0F00) >> 8);

                    for i in 0..x + 1 {
                        self.v[i] = self.memory[self.index + i];
                    }

                    self.pc += 2;
                }
                _ => panic!("Unknown opcode!"),
            },
            _ => panic!("Unknown opcode!"),
        }
    }
}

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
