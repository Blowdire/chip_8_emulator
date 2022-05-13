use super::file_utils;
use rand;
use rand::prelude::*;
use rand::rngs;
pub struct chip8 {
    registers: [u8; 16],
    memory: [u8; 4096],
    index: u16,
    pc: u16,
    stack: [u16; 16],
    sp: u8,
    delay_timer: u8,
    sound_timer: u8,
    keypad: [bool; 16],
    video: [bool; 64 * 32],
    rng: rand::rngs::ThreadRng,
}
const VIDEO_WIDTH: u16 = 64;
const VIDEO_HEIGHT: u16 = 32;
const FONTSET_START_ADDRESS: u8 = 0x50;
const FONTSET: [u8; 80] = [
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
struct OpCode {
    higher_byte: u8,
    lower_byte: u8,
}

impl OpCode {
    pub fn get_nnn(&self) -> u16 {
        let full_op_code: u16 = (self.higher_byte << 8 | self.lower_byte) as u16;
        let nnn = full_op_code & 0xFFF;
        nnn as u16
    }
}
impl chip8 {
    pub fn new() -> Self {
        let mut chip8 = Self {
            registers: [0; 16],
            pc: 0x200,
            memory: [0; 4096],
            stack: [0; 16],
            index: 0,
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: [false; 16],
            video: [false; 64 * 32],
            rng: rand::thread_rng(),
        };
        chip8.load_fontset();
        chip8
    }
    pub fn get_random_number(&mut self) -> u8 {
        let random = self.rng.gen_range(0..255);
        random
    }
    pub fn reset(&mut self) {
        self.pc = 0x200;
        self.sp = 0;
        self.delay_timer = 0;
        self.sound_timer = 0;
        self.keypad = [false; 16];
        self.video = [false; 64 * 32];
        self.registers = [0; 16];
        self.memory = [0; 4096];
        self.stack = [0; 16];
        self.index = 0;
        self.load_fontset();
    }

    fn load_fontset(&mut self) {
        self.memory[80..(80 + 80)].copy_from_slice(&FONTSET);
    }
    pub fn load_rom(&mut self, path: &str) {
        let rom_buffer = file_utils::read_file_to_buffer(path);
        let rombuffer_length = rom_buffer.len();
        self.memory[0x200..(0x200 + rombuffer_length)].copy_from_slice(&rom_buffer);
    }
    fn fetch(&mut self) -> OpCode {
        let higher_byte = self.memory[self.pc as usize];
        let lower_byte = self.memory[(self.pc + 1) as usize];
        let op_code = OpCode {
            higher_byte,
            lower_byte,
        };
        self.pc += 2;

        op_code
    }
    fn decode_and_execute(&mut self, op_code: OpCode) {
        let first_half_byte = op_code.higher_byte >> 4;
        match first_half_byte {
            0x0 => {
                match op_code.lower_byte {
                    //clear video
                    0xe0 => {
                        self.video = [false; 64 * 32];
                    }
                    //return from subroutine
                    0xee => {
                        self.sp -= 1;
                        self.pc = self.stack[self.sp as usize];
                    }
                    _ => {
                        println!("Unknown opcode: {:X}", op_code.higher_byte);
                    }
                }
            }
            //jump to nnn
            0x1 => {
                self.pc = op_code.get_nnn();
            }
            //call subroutine
            0x2 => {
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = op_code.get_nnn();
            }
            //skip next instruction if Vx = kk
            0x3 => {
                let vx: u8 = op_code.higher_byte & 0xF;
                let value = op_code.lower_byte;
                if self.registers[vx as usize] == value {
                    self.pc += 2;
                }
            }
            //skip next instruction if Vx != kk
            0x4 => {
                let vx: u8 = op_code.higher_byte & 0xF;
                let value = op_code.lower_byte;
                if self.registers[vx as usize] != value {
                    self.pc += 2;
                }
            }
            //skip next instruction if Vx = Vy
            0x5 => {
                let vx: u8 = op_code.higher_byte & 0xF;
                let vy: u8 = op_code.lower_byte >> 4;
                if self.registers[vx as usize] == self.registers[vy as usize] {
                    self.pc += 2;
                }
            }
            //set Vx = kk
            0x6 => {
                let vx: u8 = op_code.higher_byte & 0xF;
                let value = op_code.lower_byte;
                self.registers[vx as usize] = value;
            }
            //set Vx = Vx + kk
            0x7 => {
                let vx: u8 = op_code.higher_byte & 0xF;
                let value = op_code.lower_byte;
                self.registers[vx as usize] += value;
            }
            //set Vx = Vy
            0x8 => {
                match op_code.lower_byte & 0xF {
                    //set Vx = Vy
                    0x0 => {
                        let vx: u8 = op_code.higher_byte & 0xF;
                        let vy: u8 = op_code.lower_byte >> 4;
                        self.registers[vx as usize] = self.registers[vy as usize];
                    }
                    //set Vx = Vx | Vy
                    0x1 => {
                        let vx: u8 = op_code.higher_byte & 0xF;
                        let vy: u8 = op_code.lower_byte >> 4;
                        self.registers[vx as usize] |= self.registers[vy as usize];
                    }
                    //set Vx = Vx & Vy
                    0x2 => {
                        let vx: u8 = op_code.higher_byte & 0xF;
                        let vy: u8 = op_code.lower_byte >> 4;
                        self.registers[vx as usize] &= self.registers[vy as usize];
                    }
                    //set Vx = Vx ^ Vy
                    0x3 => {
                        let vx: u8 = op_code.higher_byte & 0xF;
                        let vy: u8 = op_code.lower_byte >> 4;
                        self.registers[vx as usize] ^= self.registers[vy as usize];
                    }
                    //set Vx = Vx + Vy, set VF = carry
                    0x4 => {
                        let vx: u8 = op_code.higher_byte & 0xF;
                        let vy: u8 = op_code.lower_byte >> 4;
                        let result =
                            self.registers[vx as usize] as u16 + self.registers[vy as usize] as u16;
                        if result > 255 {
                            self.registers[0xF] = 1;
                        } else {
                            self.registers[0xF] = 0;
                        }
                        self.registers[vx as usize] = (result & 0xFF) as u8;
                    }
                    //set Vx = Vx - Vy, set VF = NOT borrow
                    0x5 => {
                        let vx: u8 = op_code.higher_byte & 0xF;
                        let vy: u8 = op_code.lower_byte >> 4;
                        if self.registers[vx as usize] > self.registers[vy as usize] {
                            self.registers[0xF] = 1;
                        } else {
                            self.registers[0xF] = 0;
                        }
                        self.registers[vx as usize] -= self.registers[vy as usize];
                    }
                    //set Vx = Vx SHR 1
                    0x6 => {
                        let vx: u8 = op_code.higher_byte & 0xF;
                        self.registers[0xF] = self.registers[vx as usize] & 0x1;
                        self.registers[vx as usize] >>= 1;
                    }
                    //set Vx = Vy - Vx, set VF = NOT borrow
                    0x7 => {
                        let vx: u8 = op_code.higher_byte & 0xF;
                        let vy: u8 = op_code.lower_byte >> 4;
                        if self.registers[vy as usize] > self.registers[vx as usize] {
                            self.registers[0xF] = 1;
                        } else {
                            self.registers[0xF] = 0;
                        }
                        self.registers[vx as usize] =
                            self.registers[vy as usize] - self.registers[vx as usize];
                    }
                    //set Vx = Vx SHL 1
                    0xE => {
                        let vx: u8 = op_code.higher_byte & 0xF;
                        self.registers[0xF] = self.registers[vx as usize] >> 7;
                        self.registers[vx as usize] <<= 1;
                    }
                    _ => {
                        println!("Unknown opcode: 0x{:X}", op_code.higher_byte);
                    }
                }
            }
            //skip if Vx != Vy
            0x9 => {
                let vx: u8 = op_code.higher_byte & 0xF;
                let vy: u8 = op_code.lower_byte >> 4;
                if self.registers[vx as usize] != self.registers[vy as usize] {
                    self.pc += 2;
                }
            }
            //set I = nnn
            0xA => {
                self.index = op_code.get_nnn();
            }
            //jump to address nnn+V0
            0xB => {
                self.pc = op_code.get_nnn() + self.registers[0] as u16;
            }
            //set Vx = random byte AND kk
            0xC => {
                let vx = op_code.higher_byte & 0xF;
                let random_num = self.get_random_number();
                self.registers[vx as usize] = random_num & op_code.lower_byte;
            }
            //draw sprite at (Vx, Vy) with width = 8 and height = N
            0xD => {
                let vx: u8 = op_code.higher_byte & 0xF;
                let vy: u8 = op_code.lower_byte >> 4;
                let height: u8 = op_code.lower_byte & 0xF;

                let x_position = self.registers[vx as usize] as u16 % VIDEO_WIDTH;
                let y_position = self.registers[vy as usize] as u16 % VIDEO_HEIGHT;

                self.registers[0xF] = 0;

                for row in 0..height {
                    //for each sprite get current byte corresponding to the row
                    let sprite_byte = self.memory[(self.index + row as u16) as usize];

                    for col in 0..8 {
                        //get sprite pixel using current row and mask 10000000 shifted by col
                        let sprite_pixel: u8 = sprite_byte & (0x80 >> col);

                        //get video pixel using x and y positions
                        let video_pixel: &mut u8 = &mut self.memory[((y_position + row as u16)
                            * VIDEO_WIDTH
                            + (x_position + col))
                            as usize];
                        if sprite_pixel == 0xFF {
                            if *video_pixel == 0xFF {
                                self.registers[0xF] = 1;
                            }
                            *video_pixel ^= 0xFF;
                        }
                    }
                }
            }
            //skip if key with value of Vx is pressed
            0xE => {
                match op_code.lower_byte {
                    0x9E => {
                        let vx: u8 = op_code.higher_byte & 0xF;
                        let key = self.registers[vx as usize];

                        if self.keypad[key as usize] {
                            self.pc += 2;
                        }
                    }
                    0xA1 => {
                        let vx: u8 = op_code.higher_byte & 0xF;
                        let key = self.registers[vx as usize];

                        if !self.keypad[key as usize] {
                            self.pc += 2;
                        }
                    }
                    _ => {
                        println!(
                            "Unknown opcode: {:02x}{:02x}",
                            op_code.higher_byte, op_code.lower_byte
                        );
                    }
                };
            }
            0xF => match op_code.lower_byte {
                //wait until key is pressed and set Vx = key
                0x0A => {
                    let vx: u8 = op_code.higher_byte & 0xF;

                    let mut key_index: i32 = -1;
                    for key in 0..self.keypad.len() {
                        if self.keypad[key] {
                            key_index = key as i32;
                            break;
                        }
                    }
                    if key_index != -1 {
                        self.registers[vx as usize] = key_index as u8;
                    } else {
                        self.pc -= 2;
                    }
                }
                //set delay timer = Vx
                0x15 => {
                    let vx: u8 = op_code.higher_byte & 0xF;
                    self.delay_timer = self.registers[vx as usize];
                }
                //set sound timer = Vx
                0x18 => {
                    let vx: u8 = op_code.higher_byte & 0xF;
                    self.sound_timer = self.registers[vx as usize];
                }
                //set I = I + Vx
                0x1E => {
                    let vx: u8 = op_code.higher_byte & 0xF;
                    self.index = self.index + self.registers[vx as usize] as u16;
                }
                //set I = location of sprite for digit Vx
                0x29 => {
                    let vx: u8 = op_code.higher_byte & 0xF;
                    let digit = self.registers[vx as usize];
                    self.index = (FONTSET_START_ADDRESS + (5 * digit as u8)) as u16;
                }
                //store BCD representation of Vx in memory locations I, I+1, and I+2
                0x33 => {
                    let vx: u8 = op_code.higher_byte & 0xF;
                    let mut value = self.registers[vx as usize];
                    //ones digit
                    self.memory[(self.index + 2) as usize] = value % 10;
                    value /= 10;
                    //tens digit
                    self.memory[(self.index + 1) as usize] = value % 10;
                    value /= 10;
                    //hundreds digit
                    self.memory[self.index as usize] = value % 10;
                }
                //store registers V0 through Vx in memory starting at location I
                0x55 => {
                    let vx: u8 = op_code.higher_byte & 0xF;
                    for i in 0..=vx {
                        self.memory[(self.index + i as u16) as usize] = self.registers[i as usize];
                    }
                }
                //read registers V0 through Vx from memory starting at location I
                0x65 => {
                    let vx: u8 = op_code.higher_byte & 0xF;
                    for i in 0..=vx {
                        self.registers[i as usize] = self.memory[(self.index + i as u16) as usize];
                    }
                }
                _ => {
                    println!(
                        "Unknown opcode: {:02x}{:02x}",
                        op_code.higher_byte, op_code.lower_byte
                    );
                }
            },
            _ => {
                println!("Unknown opcode: {:X}", op_code.higher_byte);
            }
        }
        //DECREMENT TIMERS
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }
}
