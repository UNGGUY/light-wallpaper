// Generate the bindings in their own module
pub mod my_protocol {
    use wayland_client;
    // import objects from the core protocol if needed
    use wayland_client::protocol::*;

    // This module hosts a low-level representation of the protocol objects
    // you will not need to interact with it yourself, but the code generated
    // by the generate_client_code! macro will use it
    pub mod __interfaces {
        // import the interfaces from the core protocol if needed
        use wayland_client::protocol::__interfaces::*;
        use wayland_protocols::xdg::shell::client::__interfaces::XDG_POPUP_INTERFACE;
        use wayland_protocols::xdg::shell::client::__interfaces::xdg_popup_interface;

        wayland_scanner::generate_interfaces!("protocols/wlr-layer-shell-unstable-v1.xml");
    }
    use self::__interfaces::*;

    // This macro generates the actual types that represent the wayland objects of
    // your custom protocol
    use wayland_protocols::xdg::shell::client::xdg_popup;
    wayland_scanner::generate_client_code!("protocols/wlr-layer-shell-unstable-v1.xml");
}
