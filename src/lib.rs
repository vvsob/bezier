pub mod curve;
mod state;
pub mod vertex;

use std::time::SystemTime;

use state::State;
pub use vertex::Vertex;

use winit::{
    event::{Event, KeyEvent, WindowEvent},
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let window_ref = &window;

    let mut state = State::new(&window).await;

    let start_time = SystemTime::now();

    let _ = event_loop.run(move |mut event, control_flow| match event {
        Event::WindowEvent {
            ref mut event,
            window_id,
        } if window_id == state.window().id() => {
            if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                physical_key: PhysicalKey::Code(KeyCode::Escape),
                                ..
                            },
                        ..
                    } => control_flow.exit(),
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::RedrawRequested => {
                        state.update(start_time.elapsed().unwrap());
                        match state.render() {
                            Ok(_) => {}
                            Err(wgpu::SurfaceError::Lost) => {} /*state.resize(state.size)*/,
                            Err(wgpu::SurfaceError::OutOfMemory) => control_flow.exit(),
                            Err(e) => eprintln!("{:?}", e),
                        };
                        window_ref.request_redraw();
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    });
}

// #[rustfmt::skip]
// pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
//     1.0, 0.0, 0.0, 0.0,
//     0.0, 1.0, 0.0, 0.0,
//     0.0, 0.0, 0.5, 0.5,
//     0.0, 0.0, 0.0, 1.0,
// );
