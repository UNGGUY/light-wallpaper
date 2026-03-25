use winit::event_loop::{self, EventLoop};

use crate::app::App;
use crate::wayland::WayApp;

use wayland_client::Connection;

mod app;
mod context;
mod wayland;

fn main() {
    // let event_loop = EventLoop::new().unwrap();

    // event_loop.set_control_flow(event_loop::ControlFlow::Poll);

    // event_loop.run_app(&mut App::new()).unwrap();
    // Create a Wayland connection by connecting to the server through the
    // environment-provided configuration.
    let conn_res = Connection::connect_to_env();

    let conn = match conn_res {
        Ok(conn) => conn,
        Err(e) => {
            println!("{0}", e);
            return;
        }
    };
    // Retrieve the WlDisplay Wayland object from the connection. This object is
    // the starting point of any Wayland program, from which all other objects will
    // be created.
    let display = conn.display();

    // Create an event queue for our event processing
    let mut event_queue = conn.new_event_queue();
    // And get its handle to associate new objects to it
    let qh = event_queue.handle();

    // Create a wl_registry object by sending the wl_display.get_registry request.
    // This method takes two arguments: a handle to the queue that the newly created
    // wl_registry will be assigned to, and the user-data that should be associated
    // with this registry (here it is () as we don't need user-data).
    let _registry = display.get_registry(&qh, ());

    // At this point everything is ready, and we just need to wait to receive the events
    // from the wl_registry. Our callback will print the advertised globals.
    println!("Advertised globals:");

    // To actually receive the events, we invoke the `roundtrip` method. This method
    // is special and you will generally only invoke it during the setup of your program:
    // it will block until the server has received and processed all the messages you've
    // sent up to now.
    //
    // In our case, that means it'll block until the server has received our
    // wl_display.get_registry request, and as a reaction has sent us a batch of
    // wl_registry.global events.
    //
    // `roundtrip` will then empty the internal buffer of the queue it has been invoked
    // on, and thus invoke our `Dispatch` implementation that prints the list of advertised
    // globals.
    event_queue.roundtrip(&mut WayApp).unwrap();
}
