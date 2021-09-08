mod cpu;
mod display;
mod font_set;

use std::env;

use cpu::Chip8;

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let mut ch8 = Chip8::new();
    ch8.load_program(&args[1]);
    ch8.run();
}
