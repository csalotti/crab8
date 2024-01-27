use rand::{thread_rng, Rng};
use rand_distr::{Distribution, Normal};
use std::{
    hint::spin_loop,
    time::{Duration, Instant},
    usize,
};
use winit::event::ElementState;

const FONT: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub const W_HEIGHT: usize = 32;
pub const W_WIDTH: usize = 64;

const MEMORY_SIZE: usize = 4096;
const FONT_OFFSET: usize = 0x050;
const LOAD_START: usize = 0x200;

const DEFAULT_KEYS: &str = "1234qwerasdfzxcv";

#[derive(Clone, Copy, Debug)]
enum KeyState {
    Idle,
    Pressed,
}

#[derive(Debug)]
pub struct Chip8 {
    memory: [u8; MEMORY_SIZE],
    pixels: [[bool; W_WIDTH]; W_HEIGHT],
    pc: usize,
    i_register: u16,
    v_registers: [u8; 16],
    stack: Vec<usize>,
    delay_timer: u8,
    sound_timer: u8,
    keys: String,
    keys_states: [KeyState; 16],
}

#[derive(Debug)]
struct Instruction(u16, u16, u16, u16);

#[derive(PartialEq)]
pub enum Target {
    Memory,
    Pixels,
}

impl From<&Instruction> for u8 {
    fn from(instruction: &Instruction) -> u8 {
        ((instruction.2 << 4) | instruction.3) as u8
    }
}

impl From<&Instruction> for u16 {
    fn from(instruction: &Instruction) -> u16 {
        (instruction.1 << 8) | (instruction.2 << 4) | instruction.3
    }
}

impl From<&Instruction> for usize {
    fn from(instruction: &Instruction) -> usize {
        ((instruction.1 << 8) | (instruction.2 << 4) | instruction.3) as usize
    }
}

// private method
impl Chip8 {
    fn compute<F>(&mut self, source: u16, other: u16, operation: F, trigger: bool)
    where
        F: Fn(i16, i16) -> i16,
    {
        let vx = self.v_registers[source as usize] as i16;
        let vy = self.v_registers[other as usize] as i16;
        let result = operation(vx, vy);

        if (result < 0 || result > 255) && trigger {
            self.v_registers[0xf] = !(self.v_registers[0xf] == 1) as u8;
        }

        self.v_registers[source as usize] = (result & 255i16) as u8;
    }

    fn fetch(&mut self) -> Instruction {
        //fetch
        let instruction: u16 =
            ((self.memory[self.pc] as u16) << 8) | (self.memory[self.pc + 1] as u16);
        self.pc += 2;

        //decode
        let nibble_1 = ((0xf << 12) & instruction) >> 12;
        let nibble_2 = ((0xf << 8) & instruction) >> 8;
        let nibble_3 = ((0xf << 4) & instruction) >> 4;
        let nibble_4 = 0xf & instruction;

        Instruction(nibble_1, nibble_2, nibble_3, nibble_4)
    }

    fn execute(&mut self, instruction: &Instruction) {
        // Execute
        match *instruction {
            Instruction(0, 0, 0xe, 0) => self.pixels = [[false; W_WIDTH]; W_HEIGHT],
            Instruction(0, 0, 0xe, 0xe) => self.pc = self.stack.pop().unwrap(),
            Instruction(1, ..) => self.pc = usize::from(instruction),
            Instruction(2, ..) => {
                self.stack.push(self.pc);
                self.pc = usize::from(instruction)
            }
            Instruction(3, x, ..) => {
                if self.v_registers[x as usize] == u8::from(instruction) {
                    self.pc += 2
                }
            }
            Instruction(4, x, ..) => {
                if self.v_registers[x as usize] != u8::from(instruction) {
                    self.pc += 2
                }
            }
            Instruction(5, x, y, ..) => {
                if self.v_registers[x as usize] == self.v_registers[y as usize] {
                    self.pc += 2
                }
            }
            Instruction(6, x, ..) => self.v_registers[x as usize] = u8::from(instruction),
            Instruction(7, x, ..) => {
                self.v_registers[x as usize] = (((self.v_registers[x as usize] as u16)
                    + u16::from(instruction))
                    & 255u16) as u8
            }
            Instruction(8, x, y, 0) => self.v_registers[x as usize] = self.v_registers[y as usize],
            Instruction(8, x, y, 1) => self.compute(x, y, |u, v| u | v, false),
            Instruction(8, x, y, 2) => self.compute(x, y, |u, v| u & v, false),
            Instruction(8, x, y, 3) => self.compute(x, y, |u, v| u ^ v, false),
            Instruction(8, x, y, 4) => {
                self.v_registers[0xf] = 0;
                self.compute(x, y, |u, v| u + v, true);
            }
            Instruction(8, x, y, 5) => {
                self.v_registers[0xf] = 1;
                self.compute(x, y, |u, v| u - v, true);
            }
            Instruction(8, x, .., 6) => {
                self.v_registers[0xf] = self.v_registers[x as usize] & 1u8;
                self.v_registers[x as usize] >>= 1;
            }
            Instruction(8, x, y, 7) => {
                self.v_registers[0xf] = 1;
                self.compute(x, y, |u, v| v - u, true);
            }
            Instruction(8, x, .., 0xe) => {
                self.v_registers[0xf] = self.v_registers[x as usize] & 128u8;
                self.v_registers[x as usize] <<= 1;
            }
            Instruction(9, x, y, ..) => {
                if self.v_registers[x as usize] != self.v_registers[y as usize] {
                    self.pc += 2
                }
            }
            Instruction(0xa, ..) => self.i_register = u16::from(instruction),
            Instruction(0xb, ..) => {
                self.pc = (u16::from(instruction) + (self.v_registers[0] as u16)) as usize
            }
            Instruction(0xc, x, ..) => {
                let mut rng = rand::thread_rng();
                self.v_registers[x as usize] = rng.gen::<u8>() & u8::from(instruction);
            }
            Instruction(0xd, x, y, n) => self.draw(x, y, n),
            Instruction(0xe, x, 9, 0xe) => {
                if let KeyState::Pressed = self.keys_states[x as usize] {
                    self.pc += 2;
                }
            }
            Instruction(0xe, x, 0xa, 1) => {
                if let KeyState::Idle = self.keys_states[x as usize] {
                    self.pc += 2;
                }
            }
            Instruction(0xf, x, 0, 7) => self.v_registers[x as usize] = self.delay_timer,
            Instruction(0xf, x, 1, 5) => self.delay_timer = self.v_registers[x as usize],
            Instruction(0xf, x, 1, 8) => self.sound_timer = self.v_registers[x as usize],
            Instruction(0xf, x, 1, 0xe) => {
                let result = self.i_register + (self.v_registers[x as usize] as u16);
                self.v_registers[0xf] = (result > 0x0fff) as u8;
                self.i_register = result & 0xfff;
            }
            Instruction(0xf, x, 0, 0xa) => {
                if let KeyState::Idle = self.keys_states[x as usize] {
                    self.pc -= 2
                }
            } // Freeze until key pressed
            Instruction(0xf, x, 2, 9) => {
                self.i_register = (FONT_OFFSET as u16) + 5 * (self.v_registers[x as usize] as u16)
            }
            Instruction(0xf, x, 3, 3) => {
                let vx: u16 = self.v_registers[x as usize] as u16;
                for i in 0..3u32 {
                    self.memory[(self.i_register + (i as u16)) as usize] =
                        (((vx % 10u16.pow(3 - i)) / 10u16.pow(2 - i)) & 255u16) as u8;
                }
            }
            Instruction(0xf, x, 5, 5) => {
                for i in 0..=x {
                    self.memory[(self.i_register + i) as usize] = self.v_registers[i as usize]
                }
            }
            Instruction(0xf, x, 6, 5) => {
                for i in 0..=x {
                    self.v_registers[i as usize] = self.memory[(self.i_register + i) as usize]
                }
            }
            _ => panic!("Unknow instruction {:?}", instruction),
        };
    }

    fn draw(&mut self, x: u16, y: u16, n: u16) {
        // Modulo coordinates to stay in range
        let x = (self.v_registers[x as usize] & 63) as usize;
        let y = (self.v_registers[y as usize] & 31) as usize;
        let i = self.i_register;

        self.v_registers[0xf] = 0;

        for row in 0..usize::from(n) {
            let sprite = self.memory[usize::from(i) + row];
            for col in 0..8 {
                let (c_x, c_y) = (x + col, y + row);
                if (c_y < W_HEIGHT) && (c_x < W_WIDTH) {
                    let d_pixel = self.pixels[c_y][c_x];
                    let s_pixel = (((0x80 >> col) & sprite) >> (7 - col)) != 0;
                    if d_pixel && s_pixel {
                        self.v_registers[0xf] = 1;
                    }
                    self.pixels[c_y][c_x] = d_pixel ^ s_pixel;
                }
            }
        }
    }
}

// public method
impl Chip8 {
    pub fn new() -> Self {
        let mut memory = [0; 4096];
        // Fill font in memory
        for (pos, &b) in FONT.iter().enumerate() {
            memory[FONT_OFFSET + pos] = b;
        }
        Self {
            memory,
            pixels: [[false; W_WIDTH]; W_HEIGHT],
            pc: LOAD_START,
            i_register: 0u16,
            v_registers: [0u8; 16],
            stack: vec![],
            delay_timer: 0u8,
            sound_timer: 0u8,
            keys: DEFAULT_KEYS.to_string(),
            keys_states: [KeyState::Idle; 16],
        }
    }

    pub fn pixels(&self) -> Vec<Vec<bool>> {
        self.pixels
            .into_iter()
            .map(|row| row.into_iter().collect())
            .collect()
    }
    pub fn load(&mut self, instructions: &[u8]) {
        // Fill with chip 8 instrucitons
        for (pos, &b) in instructions.iter().enumerate() {
            self.memory[LOAD_START + pos] = b;
        }
    }

    pub fn update_key_states(&mut self, key: &str, state: ElementState) {
        if self.keys.contains(key) {
            let key_idx: usize = self.keys.find(key).unwrap();
            match state {
                ElementState::Pressed => self.keys_states[key_idx] = KeyState::Pressed,
                ElementState::Released => self.keys_states[key_idx] = KeyState::Idle,
            }
        }
    }

    pub fn step(&mut self) -> Target {
        let start = Instant::now();
        let instruction = self.fetch();
        let _ = self.execute(&instruction);

        // Expected time wait depending on the instruction
        let mut rng = thread_rng();
        let interval_dist = match instruction {
            Instruction(0, 0, 0xe, 0) => Normal::new(109.0, 0.0).unwrap(),
            Instruction(0, 0, 0xe, 0xe)
            | Instruction(1, ..)
            | Instruction(2, ..)
            | Instruction(0xb, ..) => Normal::new(105.0, 5.0).unwrap(),
            Instruction(3, ..) | Instruction(4, ..) | Instruction(0xa, ..) => {
                Normal::new(55.0, 9.0).unwrap()
            }
            Instruction(5, ..) | Instruction(9, ..) | Instruction(0xe, ..) => {
                Normal::new(73.0, 0.0).unwrap()
            }
            Instruction(6, ..) => Normal::new(27.0, 0.0).unwrap(),
            Instruction(7, ..)
            | Instruction(0xf, .., 0, 7)
            | Instruction(0xf, .., 1, 5)
            | Instruction(0xf, .., 1, 8) => Normal::new(45.0, 0.0).unwrap(),
            Instruction(8, ..) => Normal::new(200.0, 0.0).unwrap(),
            Instruction(0xc, ..) => Normal::new(164.0, 0.0).unwrap(),
            Instruction(0xf, .., 0, 0xa) => Normal::new(0.0, 0.0).unwrap(),
            Instruction(0xf, .., 1, 0xe) => Normal::new(86.0, 14.0).unwrap(),
            Instruction(0xf, .., 2, 9) => Normal::new(91.0, 0.0).unwrap(),
            Instruction(0xf, .., 3, 3) => Normal::new(927.0, 545.0).unwrap(),
            Instruction(0xf, .., 5 | 6, 5) => Normal::new(605.0, 477.0).unwrap(),
            Instruction(0xd, ..) => Normal::new(22734.0, 4634.0).unwrap(),
            _ => panic!("Unknow Instruction {:?}", instruction),
        };

        //let interval = Duration::from_micros(interval_dist.sample(&mut rng) as u64);
        //while start.elapsed() < interval {
        //    spin_loop()
        //}

        match instruction {
            Instruction(0xd, ..) | Instruction(0, 0, 0xe, 0) => Target::Pixels,
            _ => Target::Memory,
        }
    }
}
