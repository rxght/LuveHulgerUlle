#![allow(dead_code)]
use app::App;
use graphics::Graphics;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

mod app;
mod drawables;
mod graphics;
mod input;

fn main() {
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();

    let mut gfx = Graphics::new(window, &event_loop);

    let input = input::Input::new();
    let mut last_frame_time = std::time::Instant::now();

    let mut app = App::new(&mut gfx);

    event_loop.run(move |event, _window_target, control_flow| {
        // gui gets priority to window events
        if let Event::WindowEvent { event, .. } = &event {
            if gfx.gui().update(event) {
                return;
            }
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
            Event::RedrawRequested(_) => {
                let frame_time = std::time::Instant::now();
                let delta_time = frame_time - last_frame_time;
                last_frame_time = frame_time;

                gfx.gui().begin_frame();
                app.run(&mut gfx, &input, delta_time);
                input.clear_presses();
                if gfx.is_drawable() {
                    gfx.draw_frame()
                }
            }
            Event::MainEventsCleared => {
                gfx.get_window().request_redraw();
            }
            _ => (),
        }
    });
}
