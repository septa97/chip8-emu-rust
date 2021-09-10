use rand::Rng;

use std::fs::File;
use std::io;
use std::io::prelude::*;

pub const GFX_SIZE: usize = 2048;
pub const KEYPAD_SIZE: usize = 16;
pub const MEMORY_SIZE: usize = 4096;
pub const FONTSET_SIZE: usize = 80;
pub const WIDTH: usize = 64;

pub struct Chip8 {
    pub draw_flag: bool,
    pub gfx: [u8; GFX_SIZE],
    pub key: [bool; KEYPAD_SIZE],
    pub opcode: u16,
    pub memory: [u8; MEMORY_SIZE],
    pub v: [u8; 16],  // CPU registers
    pub index: usize, // Index register / memory address register
    pub pc: usize,    // program counter
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub stack: [usize; 16],
    pub sp: usize, // stack pointer
    pub fontset: [u8; FONTSET_SIZE],
}

impl Chip8 {
    pub fn update_timers(&mut self) {
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

    pub fn init(&mut self) {
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

    pub fn load_rom(&mut self, file_path: &String) -> Result<(), io::Error> {
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

    pub fn emulate_cycle(&mut self) {
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