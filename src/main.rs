pub mod disassembler;
pub mod file_utils;
mod chip8;
fn main() {
    let path = "src/fishie.ch8";
    let buffer = file_utils::read_file_to_buffer(path);
    let mut program_counter = 0;
    let buff_len = buffer.len();
    while program_counter < buff_len {
        program_counter = disassembler::disassemble_chip8(&buffer, program_counter);
    }
}
//
