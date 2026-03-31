use std::os::raw::c_void;

use wayland_client::{Connection, Proxy};

use context::Context;
use wayland::State;

mod app;
mod context;
mod wayland;

fn main() {
    let conn = Connection::connect_to_env().unwrap();

    let mut event_queue = conn.new_event_queue();
    let qhandle = event_queue.handle();

    let display = conn.display();
    display.get_registry(&qhandle, ());

    let mut state = State {
        running: true,
        base_surface: None,
        configured: false,
        render: false,
        context: None,
        layer_shell: None,
        output: None,
        layer_surface: None,
        width: 0,
        height: 0,
        output_scale: 1,
    };

    while state.running {
        event_queue.blocking_dispatch(&mut state).unwrap();

        if state.configured && state.context.is_none() {
            let display_ptr = conn.backend().display_ptr() as *mut c_void;

            let surface_ptr = state.base_surface.as_ref().unwrap().id().as_ptr() as *mut c_void;

            state.context = Some(
                Context::create_for_wayland(
                    surface_ptr,
                    display_ptr,
                    state.width * 2,
                    state.height * 2,
                )
                .unwrap(),
            );
        }

        if state.configured && state.render {
            if let Some(context) = state.context.as_mut() {
                context.render_wayland().unwrap();
            }
            if let Some(surface) = state.base_surface.as_ref() {
                surface.commit();
            }
            //state.render = false;
        }
    }
}
