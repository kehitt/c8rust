use c8rust::emulator::Emulator;
use winit::{dpi::LogicalSize, event::Event, event_loop::EventLoop, window::WindowBuilder};

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("CHIP-8 Emulator")
        .with_inner_size(LogicalSize::new(1080, 540))
        .build(&event_loop)
        .unwrap();

    let mut emulator = Emulator::new(&window);

    event_loop.run(move |event, _, control_flow| {
        let flow_change = match event {
            Event::WindowEvent {
                window_id, event, ..
            } if window_id == window.id() => emulator.handle_window_event(event),
            Event::MainEventsCleared => emulator.handle_update(&window),
            Event::RedrawRequested(_) => emulator.handle_redraw(),
            _ => None,
        };

        if let Some(new_control_flow) = flow_change {
            *control_flow = new_control_flow;
        }
    })
}
