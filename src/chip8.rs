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
}
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
struct op_code {
    higher_byte: u8,
    lower_byte: u8,
}

impl op_code {
    pub fn get_nnn(&self) -> u16 {
        let full_op_code = self.higher_byte << 8 | self.lower_byte;
        let nnn = full_op_code & 0x0FFF as u16;
        nnn
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
        };
        chip8.load_fontset();
        chip8
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
        self.memory[..80].copy_from_slice(&FONTSET);
    }
    pub fn load_rom(&mut self, path: &str) {
        let rom_buffer = file_utils::read_file_to_buffer(path);
        let rombuffer_length = rom_buffer.len();
        self.memory[0x200..(0x200 + rombuffer_length)].copy_from_slice(&rom_buffer);
    }
    fn fetch(&self) -> u16 {
        let higher_byte = self.memory[self.pc as usize];
        let lower_byte = self.memory[(self.pc + 1) as usize];
        let op_code = op_code {
            higher_byte,
            lower_byte,
        };
        self.pc += 2;

        op_code
    }
    fn decode_and_execute(&mut self, op_code: op_code) {
        let first_half_byte = op_code.higher_byte >> 4;
        match first_half_byte {
            0x0 => {
                match op_code.higher_byte {
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
                match op_code.higher_byte {
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
                        if (result > 255) {
                            self.registers[0xF] = 1;
                        } else {
                            self.registers[0xF] = 0;
                        }
                        self.registers[vx as usize] = result & 0xFF;
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
                }
            }
            //set Vx = Vx OR Vy
            0x9 => {}
            _ => {
                println!("Unknown opcode: {:X}", op_code.higher_byte);
            }
        }
    }
}
