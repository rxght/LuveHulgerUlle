#![allow(dead_code)]

use std::sync::Arc;

use app::App;
use graphics::Graphics;
use ui::Ui;
use winit::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
    window::Window,
};

mod app;
mod drawables;
mod graphics;
mod input;
mod ui;

fn main() {
    // initialize subsystems
    let (mut gfx, event_loop) = Graphics::new();
    let mut window_size = gfx.get_window().inner_size();

    let input = input::Input::new();
    let ui = Ui::new();
    let mut last_frame_time = std::time::Instant::now();

    let mut app = App::new(&mut gfx, input.clone(), ui.clone());

    let mut minimized = false;

    event_loop.run(move |event, _window_target, control_flow| {
        if let Event::WindowEvent {
            event: WindowEvent::Focused(false),
            ..
        } = event
        {
            println!("Window unfocused.");
            return;
        }

        if ui.handle_event(&event, input.clone(), gfx.get_window().inner_size().into()) {
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
                let window = gfx.get_window();
                let window_inner_size = window.inner_size();
                if window_inner_size != window_size {
                    app.resize_callback(&mut gfx);
                    ui.handle_resize(window_inner_size.into());
                    gfx.recreate_swapchain();

                    window_size = window_inner_size;
                    minimized = check_minimized(window);
                }
                let frame_time = std::time::Instant::now();
                let delta_time = frame_time - last_frame_time;
                last_frame_time = frame_time;
                app.run(&mut gfx, delta_time);
                ui.draw(&mut gfx);
                input.clear_presses();
                if !minimized {
                    gfx.draw_frame()
                }
            }
            _ => (),
        }
    });
}

fn check_minimized(window: Arc<Window>) -> bool {
    let extent = window.inner_size();
    return window.is_minimized().unwrap_or(false) || extent.width == 0 || extent.height == 0;
}
