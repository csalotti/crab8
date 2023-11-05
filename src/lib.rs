mod chip8;
mod render;

use chip8::{Chip8, IBM_LOGO, W_HEIGHT, W_WIDTH};
use render::Render;
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::EventLoop,
    keyboard::{Key, NamedKey},
    window::WindowBuilder,
};

pub fn run() -> Result<(), impl std::error::Error> {
    let (w_height, w_width) = (W_HEIGHT as u32, W_WIDTH as u32);

    let event_loop = EventLoop::new().unwrap();

    let window = WindowBuilder::new()
        .with_title("A fantastic window!")
        .with_inner_size(winit::dpi::LogicalSize::new(w_width, w_height))
        .build(&event_loop)
        .unwrap();

    let mut chip = Chip8::new();
    let mut render = pollster::block_on(Render::new(window));

    chip.load(&IBM_LOGO);

    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { event, window_id } if window_id == render.window().id() => {
                if !render.input(&event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    logical_key: Key::Named(NamedKey::Escape),
                                    state: ElementState::Pressed,
                                    ..
                                },
                            ..
                        } => elwt.exit(),
                        WindowEvent::RedrawRequested => {
                            // Notify the windowing system that we'll be presenting to the window.
                            match render.render(&chip.pixels()) {
                                Ok(_) => {}
                                Err(wgpu::SurfaceError::Lost) => render.resize(*render.size()),
                                Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                                Err(e) => eprintln!("Unexpeted errror :{:?}", e),
                            }
                        }
                        WindowEvent::Resized(physical_size) => {
                            render.resize(physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                            // new_inner_size is &&mut so we have to dereference it twice
                            let inner_size = render.window().inner_size();
                            render.resize(PhysicalSize {
                                width: (scale_factor * f64::from(inner_size.width)) as u32,
                                height: (scale_factor * f64::from(inner_size.height)) as u32,
                            });
                        }
                        _ => (),
                    }
                }
            }
            Event::AboutToWait => {
                render.window().request_redraw();
            }

            _ => (),
        }

        if chip.step() == chip8::Target::Pixels {
            render.window().request_redraw();
        }
    })
}
