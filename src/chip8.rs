use rand::Rng;
use sdl2::{
    audio::{AudioCallback, AudioSpecDesired},
    event::Event,
    keyboard::Keycode,
    pixels::Color,
    rect::Rect,
    render::Canvas,
    video::Window,
};

use std::{thread::sleep, time::Duration};

const ROWS: usize = 64;
const COLS: usize = 32;
const WINDOW_WIDTH: usize = 1280;
const WINDOW_HEIGHT: usize = 720;
const PIXEL_WIDTH: usize = WINDOW_WIDTH / ROWS;
const PIXEL_HEIGHT: usize = WINDOW_HEIGHT / COLS;

const FONT_SET: [u8; 80] = [
    0xf0, 0x90, 0x90, 0x90, 0xf0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xf0, 0x10, 0xf0, 0x80, 0xf0, // 2
    0xf0, 0x10, 0xf0, 0x10, 0xf0, // 3
    0x90, 0x90, 0xf0, 0x10, 0x10, // 4
    0xf0, 0x80, 0xf0, 0x10, 0xf0, // 5
    0xf0, 0x80, 0xf0, 0x90, 0xf0, // 6
    0xf0, 0x10, 0x20, 0x40, 0x40, // 7
    0xf0, 0x90, 0xf0, 0x90, 0xf0, // 8
    0xf0, 0x90, 0xf0, 0x10, 0xf0, // 9
    0xf0, 0x90, 0xf0, 0x90, 0x90, // A
    0xe0, 0x90, 0xe0, 0x90, 0xe0, // B
    0xf0, 0x80, 0x80, 0x80, 0xf0, // C
    0xe0, 0x90, 0x90, 0x90, 0xe0, // D
    0xf0, 0x80, 0xf0, 0x80, 0xf0, // E
    0xf0, 0x80, 0xf0, 0x80, 0x80, // F
];

type Display = [[u8; COLS]; ROWS];

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

pub struct Chip8 {
    pc: u16, // Program counter
    memory: [u8; 4096],

    // Registers
    v: [u8; 16],
    i: u16,

    // Timers
    dt: u8, // Delay
    st: u8, // Sound

    sp: u8, // Stack pointer
    stack: [u16; 16],

    display: Display,
    keypad: [bool; 16],

    nnn: u16,
    n: u16,
    x: usize,
    y: usize,
    kk: u8,
}

impl Chip8 {
    const START_ADDR: usize = 0x200;

    pub fn new() -> Self {
        let mut memory = [0; 4096];
        memory[..FONT_SET.len()].clone_from_slice(&FONT_SET);

        Self {
            pc: Self::START_ADDR as u16,
            memory,
            v: [0; 16],
            i: 0,
            dt: 0,
            st: 0,
            sp: 0,
            stack: [0; 16],
            display: clear_display(),
            keypad: [false; 16],

            nnn: 0,
            n: 0,
            x: 0,
            y: 0,
            kk: 0,
        }
    }

    pub fn load(&mut self, rom: &[u8]) {
        self.memory[Self::START_ADDR..(Self::START_ADDR + rom.len())].clone_from_slice(rom);
    }

    pub fn run(&mut self) {
        let mut timer_cycles = 0;

        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let audio_subsystem = sdl_context.audio().unwrap();

        let desired_spec = AudioSpecDesired {
            channels: Some(1),
            freq: Some(44_100),
            samples: None,
        };

        let sound_device = audio_subsystem
            .open_playback(None, &desired_spec, |spec| SquareWave {
                phase_inc: 440.0 / spec.freq as f32,
                phase: 0.0,
                volume: 0.25,
            })
            .unwrap();

        let window = video_subsystem
            .window("chip8", WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32)
            .position_centered()
            .build()
            .unwrap();

        let mut event_pump = sdl_context.event_pump().unwrap();
        let mut canvas = window.into_canvas().build().unwrap();

        loop {
            timer_cycles += 1;
            self.execute_opcode();
            for event in event_pump.poll_iter() {
                // set keycodes here
                match event {
                    Event::Quit { .. } => return,
                    Event::KeyDown {
                        keycode: Some(Keycode::Q),
                        ..
                    } => self.keypad[4] = true,
                    Event::KeyDown {
                        keycode: Some(Keycode::W),
                        ..
                    } => self.keypad[5] = true,
                    Event::KeyDown {
                        keycode: Some(Keycode::E),
                        ..
                    } => self.keypad[6] = true,
                    Event::KeyUp { .. } => self.reset_keypad(),
                    _ => {}
                }
            }

            canvas.set_draw_color(Color::BLACK);
            canvas.clear();
            for col in 0..COLS {
                for row in 0..ROWS {
                    if self.display[row][col] > 0 {
                        draw_pixel(row, col, &mut canvas);
                    }
                }
            }
            canvas.present();

            if self.st > 0 {
                sound_device.resume();
            } else {
                sound_device.pause();
            }
            if timer_cycles == 7 {
                self.update_timers();
                timer_cycles = 0;
            }

            sleep(Duration::from_millis(1));
        }
    }

    fn execute_opcode(&mut self) {
        let opcode = (self.memory[self.pc as usize] as u16) << 8
            | self.memory[(self.pc + 1) as usize] as u16;

        let nibbles = (
            (opcode & 0xf000) >> 12,
            (opcode & 0x0f00) >> 8,
            (opcode & 0x00f0) >> 4,
            (opcode & 0x000f),
        );

        self.x = nibbles.1 as usize;
        self.y = nibbles.2 as usize;
        self.n = nibbles.3;
        self.kk = (opcode & 0x00ff) as u8;
        self.nnn = opcode & 0x0fff;

        match nibbles {
            (0, 0, 0xe, 0) => self.inst_00e0(),
            (0, 0, 0xe, 0xe) => self.inst_00ee(),
            (1, _, _, _) => self.inst_1nnn(),
            (2, _, _, _) => self.inst_2nnn(),
            (3, _, _, _) => self.inst_3xkk(),
            (4, _, _, _) => self.inst_4xkk(),
            (5, _, _, _) => self.inst_5xy0(),
            (6, _, _, _) => self.inst_6xkk(),
            (7, _, _, _) => self.inst_7xkk(),
            (8, _, _, 0) => self.inst_8xy0(),
            (8, _, _, 1) => self.inst_8xy1(),
            (8, _, _, 2) => self.inst_8xy2(),
            (8, _, _, 3) => self.inst_8xy3(),
            (8, _, _, 4) => self.inst_8xy4(),
            (8, _, _, 5) => self.inst_8xy5(),
            (8, _, _, 6) => self.inst_8xy6(),
            (8, _, _, 7) => self.inst_8xy7(),
            (8, _, _, 0xe) => self.inst_8xye(),
            (9, _, _, 0) => self.inst_9xy0(),
            (0xa, _, _, _) => self.inst_annn(),
            (0xb, _, _, _) => self.inst_bnnn(),
            (0xc, _, _, _) => self.inst_cxkk(),
            (0xd, _, _, _) => self.inst_dxyn(),
            (0xe, _, 9, 0xe) => self.inst_ex9e(),
            (0xe, _, 0xa, 1) => self.inst_exa1(),
            (0xf, _, 0, 7) => self.inst_fx07(),
            (0xf, _, 0, 0xa) => self.inst_fx0a(),
            (0xf, _, 1, 5) => self.inst_fx15(),
            (0xf, _, 1, 8) => self.inst_fx18(),
            (0xf, _, 1, 0xe) => self.inst_fx1e(),
            (0xf, _, 2, 9) => self.inst_fx29(),
            (0xf, _, 3, 3) => self.inst_fx33(),
            (0xf, _, 5, 5) => self.inst_fx55(),
            (0xf, _, 6, 5) => self.inst_fx65(),
            _ => panic!("invalid opcode: {opcode:4x}"),
        }
    }

    fn next_opcode(&mut self) {
        self.pc += 2;
    }

    fn skip_if(&mut self, condition: bool) {
        if condition {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn update_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }
        if self.st > 0 {
            self.st -= 1;
        }
    }

    fn reset_keypad(&mut self) {
        self.keypad.iter_mut().for_each(|key| *key = false);
    }

    /// CLS
    #[inline]
    fn inst_00e0(&mut self) {
        self.display = clear_display();
        self.next_opcode();
    }

    /// RET
    #[inline]
    fn inst_00ee(&mut self) {
        self.pc = self.stack[self.sp as usize];
        self.sp -= 1;
        self.next_opcode();
    }

    /// JP addr
    #[inline]
    fn inst_1nnn(&mut self) {
        self.pc = self.nnn;
    }

    /// CALL addr
    #[inline]
    fn inst_2nnn(&mut self) {
        self.sp += 1;
        self.stack[self.sp as usize] = self.pc;
        self.pc = self.nnn;
    }

    /// SE Vx, byte
    #[inline]
    fn inst_3xkk(&mut self) {
        self.skip_if(self.v[self.x] == self.kk);
    }

    /// SNE Vx, byte    
    #[inline]
    fn inst_4xkk(&mut self) {
        self.skip_if(self.v[self.x] != self.kk);
    }

    /// SE Vx, Vy
    #[inline]
    fn inst_5xy0(&mut self) {
        self.skip_if(self.v[self.x] == self.v[self.y]);
    }

    /// LD Vx, byte
    #[inline]
    fn inst_6xkk(&mut self) {
        self.v[self.x] = self.kk;
        self.next_opcode();
    }

    /// ADD Vx, byte
    #[inline]
    fn inst_7xkk(&mut self) {
        self.v[self.x] = (self.v[self.x] as u16 + self.kk as u16) as u8;
        self.next_opcode();
    }

    /// LD Vx, Vy
    #[inline]
    fn inst_8xy0(&mut self) {
        self.v[self.x] = self.v[self.y];
        self.next_opcode();
    }

    /// OR Vx, Vy
    #[inline]
    fn inst_8xy1(&mut self) {
        self.v[self.x] |= self.v[self.y];
        self.next_opcode();
    }

    /// AND Vx, Vy
    #[inline]
    fn inst_8xy2(&mut self) {
        self.v[self.x] &= self.v[self.y];
        self.next_opcode()
    }

    /// XOR Vx, Vy
    #[inline]
    fn inst_8xy3(&mut self) {
        self.v[self.x] ^= self.v[self.y];
        self.next_opcode();
    }

    /// ADD Vx, Vy
    #[inline]
    fn inst_8xy4(&mut self) {
        let result = self.v[self.x] as u16 + self.v[self.y] as u16;
        self.v[self.x] = result as u8;
        // Vf is carry
        // Set it to 1 if Vx overflows 8bits
        self.v[0xf] = u8::from(result > u8::MAX as u16);
        self.next_opcode();
    }

    /// SUB Vx, Vy
    #[inline]
    fn inst_8xy5(&mut self) {
        self.v[0xf] = u8::from(self.v[self.x] > self.v[self.y]);
        self.v[self.x] = self.v[self.x].wrapping_sub(self.v[self.y]);
        self.next_opcode();
    }

    /// SHR Vx {, Vy}
    #[inline]
    fn inst_8xy6(&mut self) {
        self.v[0xf] = self.v[self.x] & 1;
        self.v[self.x] >>= 1;
        self.next_opcode();
    }

    /// SUBN Vx, Vy
    #[inline]
    fn inst_8xy7(&mut self) {
        self.v[0xf] = u8::from(self.v[self.y] > self.v[self.x]);
        self.v[self.x] = self.v[self.y].wrapping_sub(self.v[self.x]);
        self.next_opcode();
    }

    /// SHL Vx {, Vy}
    #[inline]
    fn inst_8xye(&mut self) {
        self.v[0xf] = self.v[self.x] & 0b1000_0000;
        self.v[self.x] <<= 1;
        self.next_opcode();
    }

    /// SNE Vx, Vy
    #[inline]
    fn inst_9xy0(&mut self) {
        self.skip_if(self.v[self.x] != self.v[self.y]);
    }

    /// LD I, addr
    #[inline]
    fn inst_annn(&mut self) {
        self.i = self.nnn;
        self.next_opcode();
    }

    /// JP V0, addr
    #[inline]
    fn inst_bnnn(&mut self) {
        self.pc = self.nnn + self.v[0] as u16;
    }

    /// RND Vx, byte
    #[inline]
    fn inst_cxkk(&mut self) {
        self.v[self.x] = rand::thread_rng().gen::<u8>() & self.kk;
        self.next_opcode();
    }

    /// DRW Vx, Vy, nibble
    #[inline]
    fn inst_dxyn(&mut self) {
        self.v[0xf] = 0;
        for byte in 0..self.n {
            let y = (self.v[self.y] as usize + byte as usize) % COLS;
            for bit in 0..8 {
                let x = (self.v[self.x] as usize + bit) % ROWS;
                let color = (self.memory[(self.i + byte) as usize] >> (7 - bit)) & 1;
                self.v[0xf] |= color & self.display[x][y];
                self.display[x][y] ^= color;
            }
        }
        self.next_opcode();
    }

    /// SKP Vx
    #[inline]
    fn inst_ex9e(&mut self) {
        self.skip_if(self.keypad[self.v[self.x] as usize]);
    }

    /// SKNP Vx
    #[inline]
    fn inst_exa1(&mut self) {
        self.skip_if(!self.keypad[self.v[self.x] as usize])
    }

    /// LD Vx, DT
    #[inline]
    fn inst_fx07(&mut self) {
        self.v[self.x] = self.dt;
        self.next_opcode();
    }

    /// LD Vx, K
    #[inline]
    fn inst_fx0a(&mut self) {
        loop {
            if self.keypad.iter().filter(|&key| !key).count() > 0 {
                break;
            }
        }
        self.v[self.x] = self.keypad[self.v[self.x] as usize] as u8;
    }

    /// LD DT, Vx
    #[inline]
    fn inst_fx15(&mut self) {
        self.dt = self.v[self.x];
        self.next_opcode();
    }

    /// LD ST, Vx
    #[inline]
    fn inst_fx18(&mut self) {
        self.st = self.v[self.x];
        self.next_opcode();
    }

    /// ADD I, Vx
    #[inline]
    fn inst_fx1e(&mut self) {
        self.i += self.v[self.x] as u16;
        self.next_opcode();
    }

    /// LD F, Vx
    #[inline]
    fn inst_fx29(&mut self) {
        self.i = self.v[self.x] as u16 * 5;
        self.next_opcode();
    }

    /// LD B, Vx
    #[inline]
    fn inst_fx33(&mut self) {
        self.memory[self.i as usize] = self.v[self.x] / 100;
        self.memory[(self.i + 1) as usize] = (self.v[self.x] % 100) / 10;
        self.memory[(self.i + 2) as usize] = self.v[self.x] % 10;
        self.next_opcode();
    }

    /// LD [I], Vx
    #[inline]
    fn inst_fx55(&mut self) {
        for i in 0..=self.x {
            self.memory[self.i as usize + i] = self.v[i];
        }
        self.next_opcode();
    }

    /// LD Vx, [I]
    #[inline]
    fn inst_fx65(&mut self) {
        for i in 0..=self.x {
            self.v[i] = self.memory[self.i as usize + i]
        }
        self.next_opcode();
    }
}

fn draw_pixel(x: usize, y: usize, canvas: &mut Canvas<Window>) {
    let pixel = Rect::new(
        (x * PIXEL_WIDTH) as i32,
        (y * PIXEL_HEIGHT) as i32,
        PIXEL_WIDTH as u32,
        PIXEL_HEIGHT as u32,
    );

    canvas.set_draw_color(Color::WHITE);
    canvas.fill_rect(pixel).unwrap();
}

const fn clear_display() -> Display {
    [[0; COLS]; ROWS]
}
