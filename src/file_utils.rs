use std::fs::File;
use std::io::BufReader;
use std::io::Read;

pub fn read_file_to_buffer(path: &str) -> Vec<u8> {
    println!("Reading file: {}", path.trim());
    let f = File::open(&path).expect("no file found");
    let mut reader = BufReader::new(f);
    let mut buffer = Vec::new();

    // Read file into vector.
    reader
        .read_to_end(&mut buffer)
        .expect("Failed to read buffer");

    // Read.
    buffer
}
