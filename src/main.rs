mod chip8;

use crate::chip8::Chip8;
use std::fs::read;

fn main() {
    let mut chip8 = Chip8::new();
    chip8.load_font();

    match read("../c8games/MAZE") {
        Ok(file) => {
            chip8.load_game(file);
            for _ in 0..5 {
                chip8.run_cycle();
            }
        }
        Err(_) => eprintln!("Cannot open file"),
    }
}
