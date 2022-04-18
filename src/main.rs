mod chip8;

use std::{env, fs};

use chip8::Chip8;

fn main() {
    // TODO: rom `HIDDEN` is not working properly
    // TODO: set all keybinds
    let mut args = env::args();
    let prog = args.next().unwrap();
    if let Some(path) = args.next() {
        let mut ch8 = Chip8::new();
        let game = fs::read(path).expect("invalid path");
        ch8.load(&game);
        ch8.run();
    } else {
        eprintln!("usage: {prog} <rom path>");
    }
}
