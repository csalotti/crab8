mod chip8;
mod display;

fn main() {
    // let _ = chip8::run();
    let _ = pollster::block_on(display::run());
}
