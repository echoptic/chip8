use rand::Rng;
use std::fs;

pub type Chip8Display = [[u8; 64]; 32];

fn empty_display() -> Chip8Display {
    [[0; 64]; 32]
}

#[allow(non_snake_case)]
pub struct Chip8 {
    PC: u16,
    memory: [u8; 4096],

    // Registers
    V: [u8; 16],
    I: u16,

    // Timers
    DT: u8, // Delay
    ST: u8, // Sound

    SP: u8, // Stack pointer
    stack: [u16; 16],

    display: Chip8Display,
    keypad: [bool; 16],
}

impl Chip8 {
    pub fn new() -> Self {
        Self {
            PC: 0x200,
            memory: [0; 4096],
            V: [0; 16],
            I: 0,
            DT: 0,
            ST: 0,
            SP: 0,
            stack: [0; 16],
            display: empty_display(),
            keypad: [false; 16],
        }
    }

    pub fn load_program(&mut self, path: &str) {
        let program = fs::read(path).expect("Failed to read file");

        for i in 0..program.len() {
            self.memory[i + 0x200] = program[i]
        }
    }

    fn render(&self) {
        println!("{:?}", self.display);
        for x in 0..32 {
            for y in 0..64 {
                if self.display[x][y] == 1 {}
            }
        }
    }

    pub fn execute(&mut self) {
        while self.PC <= self.memory.len() as u16 {
            // Combines 2 8bit parts of opcode in memory by shifitng the first by 8,
            // making place for the second one, then just combining the first one and second one
            // by using bitvise OR
            // Example:
            // memory[PC] = 11111111 (8bit)
            // 11111111 << 8 == 1111111100000000
            // 1111111100000000
            // memory[PC + 1] = 10101010
            // memory[PC] | memory[PC + 1] = 1111111110101010
            // And always advance by 2 opcodes, becuase 2 opcodes is actually one
            let opcode = (self.memory[self.PC as usize] as u16) << 8
                | self.memory[(self.PC + 1) as usize] as u16;

            println!("OPCODE: {:04x} , PC: {}", opcode, self.PC);

            // IDEA: Use crate for u4 and cast it as u4 instead of u8 (Would it be worth it?)
            //
            // This sepparates the opcode into 4 4bit parts
            // The shifting just removes the zeros on the right of every nibble
            // Example:
            // 10000 >> 4 == 1
            let nibbles = (
                (opcode & 0xf000) >> 12 as u8,
                (opcode & 0x0f00) >> 8 as u8,
                (opcode & 0x00f0) >> 4 as u8,
                (opcode & 0x000f) as u8,
            );

            let nnn = opcode & 0x0fff;
            let n = opcode & 0x000f;
            let x = nibbles.1 as usize;
            let y = nibbles.2 as usize;
            let kk = (opcode & 0x0ff) as u8;

            // TODO: Why do people do the commands in opposite order than it says on the Chip8 spec?
            match nibbles {
                // 00E0
                (0, 0, 0x0e, 0) => {
                    self.display = empty_display();
                    self.next()
                }
                // 00EE
                (0, 0, 0x0e, 0x0e) => {
                    self.PC = self.stack[self.SP as usize];
                    self.SP -= 1;
                }
                // 1NNN
                (1, _, _, _) => self.PC = opcode & 0x0fff,
                // 2NNN
                (2, _, _, _) => {
                    // current PC == self.PC + 2 (next PC) ?
                    self.SP += 1;
                    self.stack[self.ST as usize] = self.PC;
                    self.PC = opcode & 0x0fff;
                }
                // 3XNN
                (3, _, _, _) => self.skip_if(self.V[x] == kk),
                // 4XNN
                (4, _, _, _) => self.skip_if(self.V[x] != kk),
                // 5XY0
                (5, _, _, _) => self.skip_if(self.V[x] == self.V[y]),
                // 6XNN
                (6, _, _, _) => {
                    self.V[x] = kk;
                    self.next()
                }
                // 7XNN
                (7, _, _, _) => {
                    let vx = self.V[x] as u16;
                    let val = kk as u16;
                    let result = vx + val;
                    self.V[x] = result as u8;
                    self.next()
                }
                // 8XY0
                (8, _, _, 0) => {
                    self.V[x] = self.V[y];
                    self.next()
                }
                // 8XY1
                (8, _, _, 1) => {
                    self.V[x] |= self.V[y];
                    self.next()
                }
                // 8XY2
                (8, _, _, 2) => {
                    self.V[x] &= self.V[y];
                    self.next()
                }
                // 8XY3
                (8, _, _, 3) => {
                    self.V[x] ^= self.V[y];
                    self.next()
                }
                // 8XY4
                (8, _, _, 4) => {
                    self.V[x] += self.V[y];
                    // Vf is carry
                    // Set it to 1 if Vx overflows 8bits
                    self.V[0x0f] = if self.V[x] > 255 as u8 { 1 } else { 0 };
                    self.next()
                }
                // 8XY5
                (8, _, _, 5) => {
                    self.V[0x0f] = if self.V[x] > self.V[y] { 1 } else { 0 };
                    self.V[x] -= self.V[y];
                    self.next()
                }
                // 8XY6
                (8, _, _, 6) => {
                    self.V[0x0f] = self.V[x] & 0x0f;
                    self.V[x] /= 2;
                    self.next()
                }
                // 8XY7
                (8, _, _, 7) => {
                    self.V[0x0f] = if self.V[y] > self.V[x] { 1 } else { 0 };
                    self.V[x] = self.V[y] - self.V[x];
                    self.next()
                }
                // 8XYE
                (8, _, _, 0x0e) => {
                    self.V[0x0f] = (self.V[x] & 0xf0) >> 4;
                    self.next()
                }
                // 9XY0
                (9, _, _, 0) => self.skip_if(self.V[x] != self.V[y]),
                // ANNN
                (0x0a, _, _, _) => {
                    self.I = opcode & 0x0fff;
                    self.next()
                }
                // BNNN
                (0x0b, _, _, _) => self.PC = nnn + self.V[0] as u16,
                // CXNN
                (0x0c, _, _, _) => {
                    self.V[x] = rand::thread_rng().gen::<u8>() & kk;
                    self.next()
                }
                // DXYN
                (0x0d, _, _, _) => {
                    self.V[0x0f] = 0;
                    for byte in 0..n {
                        let y = (self.V[y] as usize + byte as usize) % 32;
                        for bit in 0..8 {
                            let x = (self.V[x] as usize + bit) % 64;
                            let color = (self.memory[(self.I + byte) as usize] >> (7 - bit)) & 1;
                            self.V[0x0f] |= color & self.display[y][x];
                            self.display[y][x] ^= color;
                        }
                    }
                    self.next()
                }
                // EX9E
                (0x0e, _, 9, 0x0e) => self.skip_if(self.keypad[self.V[x] as usize]),
                // EXA1
                // This is the next one to work on
                (0x0e, _, 0x0a, 1) => self.next(),
                // FX07
                (0x0f, _, 0, 7) => {
                    self.V[x] = self.DT;
                    self.next()
                }
                // FX0A
                (0x0f, _, 0, 0x0a) => (),
                // FX15
                (0x0f, _, 1, 5) => {
                    self.DT = self.V[x];
                    self.next()
                }
                // FX18
                (0x0f, _, 1, 8) => {
                    self.ST = self.V[x];
                    self.next()
                }
                // FX1E
                (0x0f, _, 1, 0x0e) => {
                    self.I += self.V[x] as u16;
                    self.next()
                }
                // FX29
                (0x0f, _, 2, 9) => {
                    self.I = self.V[x] as u16 * 5;
                    self.next()
                }
                // FX33
                (0x0f, _, 3, 3) => {
                    self.memory[self.I as usize] = self.V[x] / 100;
                    self.memory[(self.I + 1) as usize] = (self.V[x] % 100) / 10;
                    self.memory[self.I as usize] = self.V[x] % 10;
                    self.next()
                }
                // FX55
                (0x0f, _, 5, 5) => {
                    for i in 0..x + 1 {
                        self.memory[self.I as usize + i] = self.V[i];
                    }
                    self.next()
                }
                // FX65
                (0x0f, _, 6, 5) => {
                    for i in 0..x + 1 {
                        self.V[i] = self.memory[self.I as usize + i]
                    }
                    self.next()
                }
                _ => self.next(),
            }
            self.render()
        }
    }

    fn next(&mut self) {
        self.PC += 2
    }

    fn skip_if(&mut self, condition: bool) {
        if condition {
            self.PC += 4
        } else {
            self.PC += 2
        }
    }
}
