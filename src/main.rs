use chip8::Chip8;
use rand::Rng;

mod chip8;

fn main() {
    let mut ch8 = Chip8::new();
    ch8.load_program("./test_opcode.ch8");
    ch8.execute();
}
