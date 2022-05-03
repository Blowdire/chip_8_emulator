use std::fmt::format;

pub fn disassemble_chip8(code_buffer: &Vec<u8>, mut program_counter: usize) -> usize {
    let current_op_code = code_buffer[program_counter];
    // println!(
    //     "{:04x}\t{:02x}{:02x}",
    //     program_counter,
    //     code_buffer[program_counter],
    //     code_buffer[program_counter + 1]
    // );
    let first_half_byte = current_op_code >> 4;
    let assembly_code = match first_half_byte {
        0x0 => match current_op_code {
            0xe0 => String::from("CLS"),
            0xee => String::from("RET"),
            _ => format!("not impl"),
        },
        0x1 => {
            format!(
                "JMP\t${:02x}{:02x}",
                code_buffer[program_counter] & 0xf,
                code_buffer[program_counter + 1]
            )
        }
        0x2 => {
            format!(
                "CALL\t${:02x}{:02x}",
                code_buffer[program_counter] & 0xf,
                code_buffer[program_counter + 1]
            )
        }
        0x3 => {
            format!(
                "S.EQ\tV{:x}, {:02x}",
                code_buffer[program_counter] & 0xf,
                code_buffer[program_counter + 1]
            )
        }
        0x4 => {
            format!(
                "S.NEQ\tV{:x}, {:02x}",
                code_buffer[program_counter] & 0xf,
                code_buffer[program_counter + 1]
            )
        }
        0x5 => {
            println!("--------------HERE-----------------");
            format!(
                "S.EQ\tV{:x}, v{:x}",
                code_buffer[program_counter] & 0xf,
                code_buffer[program_counter + 1] >> 4
            )
        }
        0x6 => {
            format!(
                "MVI\tV{:x}, {:02x}",
                code_buffer[program_counter] & 0xf,
                code_buffer[program_counter + 1]
            )
        }
        0x7 => {
            format!(
                "ADI\tV{:x}, {:02x}",
                code_buffer[program_counter] & 0xf,
                code_buffer[program_counter + 1]
            )
        }
        0x8 => {
            let last_half_bit = code_buffer[program_counter + 1] & 0xf;
            match last_half_bit {
                0x0 => format!(
                    "MOV\tV{:x}, V{:x}",
                    code_buffer[program_counter] & 0xf,
                    code_buffer[program_counter + 1] >> 4
                ),
                0x1 => format!(
                    "OR\tV{:01x}, V{:01x}",
                    code_buffer[program_counter] & 0xf,
                    code_buffer[program_counter + 1] >> 4
                ),
                0x2 => format!(
                    "AND\tV{:x}, V{:x}",
                    code_buffer[program_counter] & 0xf,
                    code_buffer[program_counter + 1] >> 4
                ),
                0x3 => format!(
                    "XOR\tV{:x}, V{:x}",
                    code_buffer[program_counter] & 0xf,
                    code_buffer[program_counter + 1] >> 4
                ),
                0x4 => format!(
                    "ADD\tV{:x}, V{:x}",
                    code_buffer[program_counter] & 0xf,
                    code_buffer[program_counter + 1] >> 4
                ),
                0x5 => format!(
                    "SUB\tV{:x}, V{:x}",
                    code_buffer[program_counter] & 0xf,
                    code_buffer[program_counter + 1] >> 4
                ),
                0x6 => format!("SHR\tV{:x}", code_buffer[program_counter] & 0xf),
                0x7 => format!(
                    "SUBN\tV{:x}, V{:x}",
                    code_buffer[program_counter] & 0xf,
                    code_buffer[program_counter + 1] >> 4
                ),
                0xe => format!("SHL\tV{:x}", code_buffer[program_counter] & 0xf),
                _ => format!(""),
            }
        }
        0x9 => {
            format!(
                "S.NEQ\tV{:x}, V{:x}",
                code_buffer[program_counter] & 0xf,
                code_buffer[program_counter + 1] >> 4
            )
        }
        0xa => {
            format!(
                "MVI\tI, ${:01x}{:02x}",
                code_buffer[program_counter] & 0xf,
                code_buffer[program_counter + 1]
            )
        }
        0xb => {
            format!(
                "JMP\t${:01x}{:02x}(V0)",
                code_buffer[program_counter] & 0xf,
                code_buffer[program_counter + 1]
            )
        }
        0xc => {
            format!(
                "RND\tV{:x}, {:02x}",
                code_buffer[program_counter] & 0xf,
                code_buffer[program_counter + 1]
            )
        }
        0xd => {
            format!(
                "DRW\tV{:x}, V{:x}, {:01x}",
                code_buffer[program_counter] & 0xf,
                code_buffer[program_counter + 1] >> 4,
                code_buffer[program_counter + 1] & 0xf
            )
        }
        0xe => match code_buffer[program_counter + 1] {
            0x9e => format!("SKP\tV{:x}", code_buffer[program_counter] & 0xf),
            0xa1 => format!("SKNP\tV{:x}", code_buffer[program_counter] & 0xf),
            _ => format!(""),
        },
        _ => format!("not impl"),
    };
    println!("{}", assembly_code);
    program_counter += 2;
    program_counter
}
