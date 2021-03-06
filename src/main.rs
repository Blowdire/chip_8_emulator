pub mod chip8;
pub mod disassembler;
pub mod file_utils;
use chip8::OpCode;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::{self, Window};
use std::env;
use std::time::{Duration, SystemTime};
//const variables definition
const SCALE: u32 = 10 as u32;
const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;
const FRAME_TIME: u128 = 1000 / 60;
const TICKS_PER_FRAME: usize = 10;

fn main() {
    println!("{}", std::env::current_dir().unwrap().to_str().unwrap());
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Help: cargo run path/to/game");
        return;
    }
    println!("PATH: {}", args[1]);
    //setup sdl
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    video_subsystem.gl_attr().set_double_buffer(true);
    let window = video_subsystem
        .window("Chip 8 Emu", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().accelerated().build().unwrap();
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();

    //initialize emulator
    let mut chip8 = chip8::chip8::new();
    //load game rom
    chip8.load_rom(&args[1]);
    //run emulator
    'running: loop {
        let frame_start_time = SystemTime::now();
        // let initial_cycle_time = SystemTime::now();
        for evt in event_pump.poll_event() {
            match evt {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'running;
                }
                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    if let Some(k) = key2btn(key) {
                        chip8.keypress(k, true)
                    }
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    if let Some(k) = key2btn(key) {
                        chip8.keypress(k, false)
                    }
                }
                _ => {}
            }
        }
        // let final_cycle_time = SystemTime::now();
        // let diff = initial_cycle_time.duration_since(final_cycle_time).unwrap();
        // if diff.as_millis() < FRAME_TIME as u128 {
        //     let difference: u64 = (FRAME_TIME - diff.as_millis()) as u64;
        //     std::thread::sleep(Duration::from_millis(difference));
        // }
        for _ in 0..TICKS_PER_FRAME {
            chip8.emulate_cycle();
        }
        chip8.tick_timers();
        //draw to window
        draw_screen(&chip8, &mut canvas);
        let frame_end_time = SystemTime::now();
        let dif = frame_end_time
            .duration_since(frame_start_time)
            .unwrap()
            .as_millis();
        if dif < FRAME_TIME {
            // std::thread::sleep(Duration::from_millis((FRAME_TIME - dif) as u64));
            sdl_context
                .timer()
                .unwrap()
                .delay(FRAME_TIME as u32 - dif as u32);
        }
    }
}
// fn main() {
//     let test = OpCode {
//         higher_byte: 0xA2,
//         lower_byte: 0xec,
//     };
//     println!("Test opcode struct: {:01x}", test.lower_byte >> 4);
// }
fn draw_screen(chip8: &chip8::chip8, canvas: &mut Canvas<Window>) {
    //clear canvas
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    let screen_buf = chip8.get_display();
    //draw with white color
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for (i, pixel) in screen_buf.iter().enumerate() {
        if *pixel {
            //convert i array wich is row to 2d
            let x = (i % SCREEN_WIDTH) as u32;
            let y = (i / SCREEN_WIDTH) as u32;
            let rect = Rect::new((x * SCALE) as i32, (y * SCALE) as i32, SCALE, SCALE);
            canvas.fill_rect(rect).unwrap();
        }
    }
    canvas.present();
}
//
fn key2btn(key: Keycode) -> Option<usize> {
    match key {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0xC),
        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xD),
        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE),
        Keycode::Z => Some(0xA),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),
        _ => None,
    }
}
