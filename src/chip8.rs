use rand::thread_rng;
use rand_distr::{Distribution, Normal};
use std::{
    hint::spin_loop,
    time::{Duration, Instant},
};

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

const IBM_LOGO: [u8; 132] = [
    0x00, 0xe0, 0xa2, 0x2a, 0x60, 0x0c, 0x61, 0x08, 0xd0, 0x1f, 0x70, 0x09, 0xa2, 0x39, 0xd0, 0x1f,
    0xa2, 0x48, 0x70, 0x08, 0xd0, 0x1f, 0x70, 0x04, 0xa2, 0x57, 0xd0, 0x1f, 0x70, 0x08, 0xa2, 0x66,
    0xd0, 0x1f, 0x70, 0x08, 0xa2, 0x75, 0xd0, 0x1f, 0x12, 0x28, 0xff, 0x00, 0xff, 0x00, 0x3c, 0x00,
    0x3c, 0x00, 0x3c, 0x00, 0x3c, 0x00, 0xff, 0x00, 0xff, 0xff, 0x00, 0xff, 0x00, 0x38, 0x00, 0x3f,
    0x00, 0x3f, 0x00, 0x38, 0x00, 0xff, 0x00, 0xff, 0x80, 0x00, 0xe0, 0x00, 0xe0, 0x00, 0x80, 0x00,
    0x80, 0x00, 0xe0, 0x00, 0xe0, 0x00, 0x80, 0xf8, 0x00, 0xfc, 0x00, 0x3e, 0x00, 0x3f, 0x00, 0x3b,
    0x00, 0x39, 0x00, 0xf8, 0x00, 0xf8, 0x03, 0x00, 0x07, 0x00, 0x0f, 0x00, 0xbf, 0x00, 0xfb, 0x00,
    0xf3, 0x00, 0xe3, 0x00, 0x43, 0xe0, 0x00, 0xe0, 0x00, 0x80, 0x00, 0x80, 0x00, 0x80, 0x00, 0x80,
    0x00, 0xe0, 0x00, 0xe0,
];
pub const W_HEIGHT: usize = 32;
pub const W_WIDTH: usize = 65;

const MEMORY_SIZE: usize = 4096;
const FONT_OFFSET: usize = 0x050;
const LOAD_START: usize = 0x200;

#[derive(Debug)]
struct Chip8 {
    memory: [u8; MEMORY_SIZE],
    display: [[bool; W_WIDTH]; W_HEIGHT],
    pc: usize,
    i_register: u16,
    v_registers: [u8; 16],
    // let mut stack: Vec<u16> = vec![];
    // let mut delay_timer: u8 = 0u8;
    // let mut sound_time: u8 = 0u8;
}

#[derive(Debug)]
struct Instruction(u16, u16, u16, u16);

impl Chip8 {
    pub fn new() -> Self {
        let mut memory = [0; 4096];
        // Fill font in memory
        for (pos, &b) in FONT.iter().enumerate() {
            memory[FONT_OFFSET + pos] = b;
        }

        Self {
            memory,
            display: [[false; W_WIDTH]; W_HEIGHT],
            pc: LOAD_START,
            i_register: 0u16,
            v_registers: [0u8; 16],
            // let mut stack: Vec<u16> = vec![];
            // let mut delay_timer: u8 = 0u8;
            // let mut sound_time: u8 = 0u8;
        }
    }

    pub fn load<I>(&mut self, instructions: I)
    where
        I: Iterator<Item = u8>,
    {
        // Fill with chip 8 instrucitons
        for (pos, b) in instructions.enumerate() {
            self.memory[LOAD_START + pos] = b;
        }
    }

    pub fn fetch(&mut self) -> Instruction {
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

    pub fn execute(&mut self, instruction: &Instruction) {
        // Execute
        match *instruction {
            Instruction(0, 0, 0xe, 0) => self.display = [[false; W_WIDTH]; W_HEIGHT],
            Instruction(1, n2, n3, n4) => self.pc = ((n2 << 8) | (n3 << 4) | n4) as usize,
            Instruction(6, x, n3, n4) => self.v_registers[x as usize] = ((n3 << 4) | n4) as u8,
            Instruction(7, x, n3, n4) => self.v_registers[x as usize] += ((n3 << 4) | n4) as u8,
            Instruction(0xa, n2, n3, n4) => self.i_register = (n2 << 8) | (n3 << 4) | n4,
            Instruction(0xd, x, y, n) => draw(
                &mut self.display,
                &self.memory,
                self.i_register,
                self.v_registers[x as usize],
                self.v_registers[y as usize],
                n,
                &mut self.v_registers[0xf],
            ),
            _ => panic!("Unknow instruction {:?}", instruction),
        };
    }

    pub fn step(&mut self) {
        let start = Instant::now();
        let instruction = self.fetch();
        let _ = self.execute(&instruction);

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

        let interval = Duration::from_micros(interval_dist.sample(&mut rng) as u64);
        while start.elapsed() < interval {
            spin_loop()
        }
    }
}

fn draw(
    display: &mut [[bool; W_WIDTH]; W_HEIGHT],
    memory: &[u8; MEMORY_SIZE],
    i: u16,
    x: u8,
    y: u8,
    n: u16,
    flag: &mut u8,
) {
    let x = (x & 63) as usize;
    let y = (y & 31) as usize;
    *flag = 0;

    for row in 0..usize::from(n) {
        let sprite = memory[usize::from(i) + row];
        for col in 0..8 {
            let (c_x, c_y) = (x + col, y + row);
            if (c_y < W_HEIGHT) && (c_x < W_WIDTH) {
                let d_pixel = display[c_y][c_x];
                let s_pixel = (((0x80 >> col) & sprite) >> (7 - col)) != 0;
                if d_pixel && s_pixel {
                    *flag = 1;
                }
                display[c_y][c_x] = d_pixel ^ s_pixel;
            }
        }
    }

    println!(
        "{}",
        display
            .iter()
            .map(|&row| format!(
                "{}\n",
                row.iter()
                    .map(|&p| if p { "#" } else { " " })
                    .collect::<String>()
            ))
            .collect::<String>()
    );
}

pub fn run() {
    let mut chip = Chip8::new();
    chip.load(IBM_LOGO.into_iter());
    loop {
        chip.step();
    }
}
