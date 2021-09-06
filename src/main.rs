use chip8::Chip8;

mod chip8;

fn main() {
    let mut ch8 = Chip8::new();
    ch8.load_program("./roms/INVADERS");
    ch8.execute();
}
