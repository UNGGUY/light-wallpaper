use winit::event_loop::{self, EventLoop};

use crate::app::App;

mod app;
fn main() {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(event_loop::ControlFlow::Poll);

    event_loop.run_app(&mut App::new()).unwrap();
}
