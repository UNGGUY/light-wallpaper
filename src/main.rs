use std::os::raw::c_void;

use std::path::Path;

use std::time::Instant;

use wayland_client::{Connection, Proxy};

use context::Context;
use wallpaper::Manager;
use wayland::State;

mod context;
mod wallpaper;
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

    let directory = Path::new("assets/wallpapers/");

    let mut manager = Manager::new(directory, 15).unwrap();

    let mut switch = false;
    let mut animation_start_time: Option<Instant> = None;
    let mut first = false;

    while state.running {
        event_queue.blocking_dispatch(&mut state).unwrap();

        if state.configured && state.context.is_none() {
            let display_ptr = conn.backend().display_ptr() as *mut c_void;

            let surface_ptr = state.base_surface.as_ref().unwrap().id().as_ptr() as *mut c_void;

            let first_path = manager.first().unwrap();

            state.context = Some(
                Context::create_for_wayland(
                    surface_ptr,
                    display_ptr,
                    state.width * (state.output_scale.max(1) as u32),
                    state.height * (state.output_scale.max(1) as u32),
                    first_path,
                )
                .unwrap(),
            );
        }
        if state.configured && state.render {
            if let Some(context) = state.context.as_mut() {
                if !switch {
                    if let Some(path) = manager.update() {
                        switch = true;
                        first = true;
                        context.reload_texture(path).unwrap();
                    }
                }
                if switch {
                    if animation_start_time.is_none() {
                        animation_start_time = Some(Instant::now());
                    }

                    // 2. 计算当前的渐变进度 (progress)
                    let elapsed = animation_start_time.unwrap().elapsed();
                    let raw_progress = (elapsed.as_secs_f32() / 1.0).min(1.0); // 假设动画总时长为 1.0 秒
                    //
                    let t = raw_progress; // 假设 raw_progress 是 f32
                    let smooth_progress = if t < 0.5 {
                        2.0_f32 * t * t
                    } else {
                        // 注意：这里的 -2.0 和 2.0 也要加上 _f32 后缀
                        1.0_f32 - (-2.0_f32 * t + 2.0_f32).powi(2) / 2.0_f32
                    };

                    context.switch(smooth_progress, first).unwrap();

                    if first {
                        first = false;
                    }

                    if smooth_progress >= 1.0 {
                        switch = false;
                        animation_start_time = None;
                    }
                }
                context.render_wayland().unwrap();
            }
            if let Some(surface) = state.base_surface.as_ref() {
                surface.commit();
            }
            //state.render = false;
        }
    }
}
