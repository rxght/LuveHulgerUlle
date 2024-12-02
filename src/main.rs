#![allow(dead_code)]
use app::App;
use graphics::Graphics;
use winit::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};

mod app;
mod drawables;
mod graphics;
mod input;

fn main() {
    // initialize subsystems
    let (mut gfx, event_loop) = Graphics::new();

    let input = input::Input::new();
    let mut last_frame_time = std::time::Instant::now();

    let mut app = App::new(&mut gfx);

    event_loop.run(move |event, _window_target, control_flow| {
        if gfx.handle_event(&input, &event) {
            return;
        }

        if input.handle_event(&event, gfx.get_window()) {
            return;
        }

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::RedrawEventsCleared => {
                let frame_time = std::time::Instant::now();
                let delta_time = frame_time - last_frame_time;
                last_frame_time = frame_time;
                gfx.clear_last_frame();
                app.run(&mut gfx, &input, delta_time);
                input.clear_presses();
                if gfx.is_drawable() {
                    gfx.draw_frame()
                }
            }
            _ => (),
        }
    });
}
