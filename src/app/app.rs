use crate::app::context;

use super::Context;
use anyhow::Result;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowAttributes;
use winit::{application::ApplicationHandler, window::Window};

pub struct App {
    window: Option<Window>,
    context: Option<Context>,
}

impl App {
    pub fn new() -> Self {
        Self {
            window: Default::default(),
            context: Default::default(),
        }
    }

    fn create_window(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        self.window = Some(event_loop.create_window(WindowAttributes::default())?);
        Ok(())
    }

    fn create_context(&mut self) -> Result<()> {
        self.context = Some(Context::create(self.window.as_ref().unwrap()).unwrap());
        Ok(())
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.create_window(event_loop).unwrap();
        self.create_context().unwrap();
        println!("create success")
    }
    #[allow(unused_variables)]
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Some(context) = self.context.as_mut() {
                    if let Some(window) = self.window.as_ref() {
                        if let Err(e) = context.render(window) {
                            println!("rander error")
                        }
                    }
                }
                println!("redraw");
            }
            _ => (),
        }
    }
}
