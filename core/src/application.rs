use std::sync::{Arc, Mutex};

use winit::{event::{KeyEvent, WindowEvent}, event_loop::{ActiveEventLoop, EventLoop}, keyboard::{Key, PhysicalKey}, window::{Window, WindowAttributes}};

use crate::{application::{event::{ApplicationEvent, ApplicationSignal}, input::{Input, KeyboardKeyState}}, assets::{texture::Texture2D, AssetsManager, AssetsManagerRef}, graphics::GraphicsContext, Timer};

pub mod event;
pub mod input;

pub trait ApplicationHandler {
    fn init(context: &GraphicsContext, assets_manager: AssetsManagerRef) -> Self;
    fn update(&mut self, dt: f32) -> ApplicationSignal;
    fn draw(&mut self, context: &GraphicsContext) -> Result<(), wgpu::SurfaceError>;
    fn handle_event(&mut self, event: ApplicationEvent) -> ApplicationSignal;
    fn handle_input(&mut self, input: &Input) -> ApplicationSignal;
}


pub struct Application<Handler: ApplicationHandler> {
    handler: Option<Handler>,
    data: Option<AppData>,
    input: Input,


    timer: Timer,
}

impl<Handler: ApplicationHandler> Application<Handler> {
    pub fn new() -> Self {

        Self {
            handler: None,
            data: None,
            input: Input::new(),
            
            timer: Timer::new(),
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

        self.handler = Some(Handler::init(&data.context, data.assets_manager.clone()));

        self.data = Some(data);

        self.timer.restart();
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {

        let data = self.data.as_mut().unwrap();
        let handler = self.handler.as_mut().unwrap();

        let elapsed = self.timer.restart();
        let elapsed_as_secs = elapsed.as_secs_f32();


        self.input.keyboard_input.set_released_keys_to_idle();
        
        let signal = match event {
            WindowEvent::CloseRequested => {event_loop.exit(); None}
            WindowEvent::RedrawRequested => {

                let signal = handler.update(elapsed_as_secs);

                match handler.draw(&data.context) {
                    Ok(()) => (),
                    Err(wgpu::SurfaceError::Outdated | wgpu::SurfaceError::Lost) => {
                        data.context.resize_surface(data.context.config.width, data.context.config.height);
                    }

                    Err(e) => log::error!("Error while drawing to surface {e:?}"),
                }

                data.window.request_redraw();

                Some(signal)
            }

            WindowEvent::Resized(size) => {
                data.context.resize_surface(size.width, size.height);

                Some(handler.handle_event(ApplicationEvent::Resized { width: size.width, height: size.height }))
            }

            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    physical_key: PhysicalKey::Code(key),
                    logical_key,
                    state,
                    ..
                },
                ..
            } => {

                let key_symbole = 
                    if let Key::Character(sym_str) = logical_key {
                    let symbole = sym_str.chars().next().unwrap();
                    Some(symbole)
                }
                else {
                    None
                };

                let key_state = match state {
                    winit::event::ElementState::Pressed => KeyboardKeyState::Pressed,
                    winit::event::ElementState::Released => KeyboardKeyState::Released,
                };

                self.input.keyboard_input.set_key_state(key, key_symbole, key_state);

                None
            }

            ev => if let Some(app_event) = ApplicationEvent::from_window_event(ev) {
                Some(handler.handle_event(app_event))
            }
            else {
                None
            }
        };


        let signal = signal
            .unwrap_or(
                handler.handle_input(&self.input)
            );
            
        self.handle_signal(event_loop, signal);

    }
}

struct AppData {
    window: Arc<Window>,
    context: GraphicsContext<'static>,
    assets_manager: AssetsManagerRef
}

impl AppData {
    async fn new(window: Window) -> Self {
        log::info!("init app data");
        let window = Arc::new(window);

        let size = window.inner_size();

        let context = GraphicsContext::new(window.clone(), size.width, size.height).await;

        let assets_manager = AssetsManager::new()
            .register_assets_type::<Texture2D>()
            ;

        Self {
            window,
            context,
            assets_manager: Arc::new(Mutex::new(assets_manager))
        }
    }
}