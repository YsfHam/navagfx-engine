use std::sync::Arc;

use winit::{event::WindowEvent, event_loop::{ActiveEventLoop, EventLoop}, window::{Window, WindowAttributes}};

use crate::{application::event::{ApplicationEvent, ApplicationSignal}, graphics::GraphicsContext};

pub mod event;

pub trait ApplicationHandler {
    fn init(&mut self, context: &GraphicsContext);
    fn update(&mut self) -> ApplicationSignal;
    fn draw(&mut self, context: &GraphicsContext);
    fn handle_event(&mut self, event: ApplicationEvent) -> ApplicationSignal;
}


pub struct Application<Handler: ApplicationHandler> {
    handler: Handler,
    data: Option<AppData>,
}

impl<Handler: ApplicationHandler> Application<Handler> {
    pub fn new(handler: Handler) -> Self {
        Self {
            handler,
            data: None,
        }
    }

    pub fn run(mut self) {
        log::info!("Application is running ...");
        
        let event_loop = EventLoop::with_user_event().build().unwrap();

        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

        event_loop.run_app(&mut self).unwrap();
    }


    fn handle_signal(&self, event_loop: &ActiveEventLoop, signal: ApplicationSignal) {
        match signal {
            ApplicationSignal::Exit => event_loop.exit(),
            ApplicationSignal::Continue => (),
        }
    }
}


impl<Handler: ApplicationHandler> winit::application::ApplicationHandler<AppData> for Application<Handler> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        log::info!("Initializing application data and handler");
        
        let window = event_loop.create_window(WindowAttributes::default()).unwrap();
        let data = smol::block_on(AppData::new(window));

        self.handler.init(&data.context);

        self.data = Some(data);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {

        let data = match &mut self.data {
            Some(data) => data,
            None => {
                log::debug!("App data is none");
                return;
            },
        };

        
        let signal = match event {
            WindowEvent::CloseRequested => {event_loop.exit(); None}
            WindowEvent::RedrawRequested => {

                let signal = self.handler.update();

                self.handler.draw(&data.context);

                data.window.request_redraw();

                Some(signal)
            }

            WindowEvent::Resized(size) => {
                data.context.resize_surface(size.width, size.height);

                Some(self.handler.handle_event(ApplicationEvent::Resized { width: size.width, height: size.height }))
            }

            ev => if let Some(app_event) = ApplicationEvent::from_window_event(ev) {
                Some(self.handler.handle_event(app_event))
            }
            else {
                None
            }
        };
        if let Some(signal) = signal {
            self.handle_signal(event_loop, signal);
        }

    }
}

struct AppData {
    window: Arc<Window>,
    context: GraphicsContext<'static>,
}

impl AppData {
    async fn new(window: Window) -> Self {
        log::info!("init app data");
        let window = Arc::new(window);

        let size = window.inner_size();

        let context = GraphicsContext::new(window.clone(), size.width, size.height).await;
        Self {
            window,
            context
        }
    }
}