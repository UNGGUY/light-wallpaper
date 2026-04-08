#![allow(unused)]
use wayland_client::{
    Connection, Dispatch, Proxy, QueueHandle, WEnum, delegate_noop,
    protocol::{
        wl_buffer, wl_compositor, wl_keyboard, wl_output, wl_registry, wl_seat, wl_shm,
        wl_shm_pool, wl_surface,
    },
};

use crate::wayland::wlr_layer_shell::my_protocol::{
    zwlr_layer_shell_v1::{Layer, ZwlrLayerShellV1},
    zwlr_layer_surface_v1::{self, Anchor, ZwlrLayerSurfaceV1},
};

use crate::context::Context;

pub struct State {
    pub(crate) running: bool,
    pub(crate) base_surface: Option<wl_surface::WlSurface>,
    pub(crate) configured: bool,
    pub(crate) render: bool,
    pub(crate) context: Option<Context>,
    pub(crate) layer_shell: Option<ZwlrLayerShellV1>,
    pub(crate) layer_surface: Option<ZwlrLayerSurfaceV1>,
    pub(crate) output: Option<wl_output::WlOutput>,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) output_scale: i32,
}

impl Dispatch<wl_registry::WlRegistry, ()> for State {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name, interface, ..
        } = event
        {
            match &interface[..] {
                "wl_compositor" => {
                    let compositor =
                        registry.bind::<wl_compositor::WlCompositor, _, _>(name, 4, qh, ());
                    let surface = compositor.create_surface(qh, ());
                    state.base_surface = Some(surface);

                    state.init_layer_background(qh);
                }
                "wl_seat" => {
                    registry.bind::<wl_seat::WlSeat, _, _>(name, 1, qh, ());
                }
                "zwlr_layer_shell_v1" => {
                    let ls = registry.bind::<ZwlrLayerShellV1, _, _>(name, 1, qh, ());
                    state.layer_shell = Some(ls);

                    state.init_layer_background(qh);
                }
                "wl_output" => {
                    let output = registry.bind::<wl_output::WlOutput, _, _>(name, 4, qh, ());
                    state.output = Some(output);
                    state.init_layer_background(qh);
                }
                _ => {}
            }
        }
    }
}

// Ignore events from these object types in this example.
delegate_noop!(State: ignore wl_compositor::WlCompositor);
delegate_noop!(State: ignore wl_surface::WlSurface);
delegate_noop!(State: ignore wl_shm::WlShm);
delegate_noop!(State: ignore wl_shm_pool::WlShmPool);
delegate_noop!(State: ignore wl_buffer::WlBuffer);

impl State {
    fn init_layer_background(&mut self, qh: &QueueHandle<State>) {
        if self.layer_shell.is_none() || self.base_surface.is_none() || self.output.is_none() {
            return;
        }

        if self.layer_surface.is_some() {
            return;
        }

        let layer_shell = self.layer_shell.as_ref().unwrap(); // ZwlrLayerShellV1
        let base_surface = self.base_surface.as_ref().unwrap(); // WlSurface
        let output = self.output.as_ref().unwrap(); // WlOutput

        // 创建 layer_surface，指定为背景层
        let layer_surface = layer_shell.get_layer_surface(
            base_surface,
            Some(output),
            Layer::Background,
            "wallpaper".into(),
            qh,
            (),
        );

        // 铺满屏幕
        layer_surface.set_anchor(Anchor::Top | Anchor::Bottom | Anchor::Left | Anchor::Right);
        // 大小设为 0，让 compositor 根据 output 原生分辨率发送 configure
        layer_surface.set_size(0, 0);

        // 临时 hack：强制 2x 过采样，测试 fractional scaling 是否是锯齿来源
        base_surface.set_buffer_scale(self.output_scale.max(1));

        // 提交 surface
        base_surface.commit();

        self.layer_surface = Some(layer_surface);
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for State {
    fn event(
        _: &mut Self,
        seat: &wl_seat::WlSeat,
        event: wl_seat::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_seat::Event::Capabilities {
            capabilities: WEnum::Value(capabilities),
        } = event
        {
            if capabilities.contains(wl_seat::Capability::Keyboard) {
                seat.get_keyboard(qh, ());
            }
        }
    }
}

impl Dispatch<wl_keyboard::WlKeyboard, ()> for State {
    fn event(
        state: &mut Self,
        _: &wl_keyboard::WlKeyboard,
        event: wl_keyboard::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_keyboard::Event::Key { key, .. } = event {
            if key == 1 {
                // ESC key
                state.running = false;
            }
        }
    }
}

impl Dispatch<ZwlrLayerShellV1, ()> for State {
    fn event(
        state: &mut Self,
        proxy: &ZwlrLayerShellV1,
        event: <ZwlrLayerShellV1 as Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwlrLayerSurfaceV1, ()> for State {
    fn event(
        state: &mut Self,
        proxy: &ZwlrLayerSurfaceV1,
        event: <ZwlrLayerSurfaceV1 as Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        if let zwlr_layer_surface_v1::Event::Configure {
            serial,
            width,
            height,
        } = event
        {
            proxy.ack_configure(serial);
            state.width = width;
            state.height = height;
            state.configured = true;
            state.render = true;
            let surface = state.base_surface.as_ref().unwrap();
        }
    }
}

impl Dispatch<wl_output::WlOutput, ()> for State {
    fn event(
        state: &mut Self,
        proxy: &wl_output::WlOutput,
        event: <wl_output::WlOutput as Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        if let wl_output::Event::Scale { factor } = event {
            state.output_scale = factor;

            if let Some(surface) = state.base_surface.as_ref() {
                surface.set_buffer_scale(factor);
                surface.commit();
            }
        }
    }
}
