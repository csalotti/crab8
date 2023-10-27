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
const MEMORY_SIZE: usize = 4096;
const FONT_OFFSET: usize = 0x050;
const LOAD_START: usize = 0x200;
const W_HEIGHT: usize = 32;
const W_WIDTH: usize = 64;

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
                    .map(|&p| if p { " " } else { "#" })
                    .collect::<String>()
            ))
            .collect::<String>()
    );
}

fn main() {
    // Hardware
    let mut memory: [u8; 4096] = [0; 4096];
    let mut display: [[bool; W_WIDTH]; W_HEIGHT] = [[false; W_WIDTH]; W_HEIGHT];
    let mut pc: usize = LOAD_START;
    let mut i_register: u16 = 0u16;
    let mut v_registers: [u8; 16] = [0u8; 16];
    // let mut stack: Vec<u16> = vec![];
    // let mut delay_timer: u8 = 0u8;
    // let mut sound_time: u8 = 0u8;

    // Fill font in memory
    for (pos, &b) in FONT.iter().enumerate() {
        memory[FONT_OFFSET + pos] = b;
    }

    // Fill with chip 8 instrucitons
    for (pos, &b) in IBM_LOGO.iter().enumerate() {
        memory[LOAD_START + pos] = b;
    }

    loop {
        // fetch
        let instruction: u16 = ((memory[pc] as u16) << 8) | (memory[pc + 1] as u16);
        pc += 2;

        // decode
        let nibble_1 = ((0xf << 12) & instruction) >> 12;
        let nibble_2 = ((0xf << 8) & instruction) >> 8;
        let nibble_3 = ((0xf << 4) & instruction) >> 4;
        let nibble_4 = 0xf & instruction;

        // Execute
        match (nibble_1, nibble_2, nibble_3, nibble_4) {
            (0, 0, 0xe, 0) => display = [[false; W_WIDTH]; W_HEIGHT],
            (1, n2, n3, n4) => pc = ((n2 << 8) | (n3 << 4) | n4) as usize, // Jump
            (6, x, n3, n4) => v_registers[x as usize] = ((n3 << 4) | n4) as u8, // set register Vx
            (7, x, n3, n4) => v_registers[x as usize] += ((n3 << 4) | n4) as u8, // add value to register Vx
            (0xa, n2, n3, n4) => i_register = (n2 << 8) | (n3 << 4) | n4, // set index register I
            (0xd, x, y, n) => draw(
                &mut display,
                &memory,
                i_register,
                v_registers[x as usize],
                v_registers[y as usize],
                n,
                &mut v_registers[0xf],
            ), // display/draw
            _ => panic!(
                "Unknow instruction {} {} {} {}",
                nibble_1, nibble_2, nibble_3, nibble_4
            ),
        };
    }
}
