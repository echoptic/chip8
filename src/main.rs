mod chip8;

use std::{env, fs};

use chip8::Chip8;

fn main() {
    let path = env::args()
        .skip(1)
        .next()
        .unwrap_or("roms/INVADERS".to_string());
    let mut ch8 = Chip8::new();
    let rom = fs::read(path).expect("invalid path");
    ch8.load(&rom);
    ch8.run();
}
