use game_engine::Engine;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    // Create an event loop
    let event_loop = EventLoop::new();

    // Build the window
    let window = WindowBuilder::new()
        .with_title("Game Engine")
        .build(&event_loop)
        .expect("Failed to create window");

    // Initialize the engine
    let mut engine = Engine::new(window);

    // Run the event loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::RedrawRequested(_) => {
                engine.update();
                engine.render();
            }
            Event::MainEventsCleared => {
                engine.window.request_redraw();
            }
            Event::WindowEvent { event, .. } => {
                if !engine.handle_window_event(&event) {
                    // If the engine didn't handle it, close on request
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    });
}
