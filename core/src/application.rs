use std::sync::{Arc, RwLock};

use winit::{dpi::LogicalSize, event::{KeyEvent, WindowEvent}, event_loop::{ActiveEventLoop, EventLoop}, keyboard::{Key, PhysicalKey}, window::{Window, WindowAttributes, WindowButtons}};

use crate::{application::{event::{ApplicationEvent, ApplicationSignal}, input::{Input, KeyboardKeyState}}, assets::{loaders::Texture2DLoader, texture::{RawRgbaImageData, Texture2D}, AssetsManager, AssetsManagerRef}, graphics::GraphicsContext, Timer};

pub mod event;
pub mod input;


pub type GraphicsContextRef<'a> = Arc<RwLock<GraphicsContext<'a>>>;

#[derive(Default, Clone)]
pub struct ApplicationSettings<'a> {
    pub window_title: &'a str,
    pub window_width: u32,
    pub window_height: u32,
    pub window_resizable: bool,
}

impl ApplicationSettings<'_> {
    fn create_window_attributes(&self) -> WindowAttributes {

        let mut window_enabled_buttons = WindowButtons::CLOSE | WindowButtons::MINIMIZE;
        if self.window_resizable {
            window_enabled_buttons |= WindowButtons::MAXIMIZE;
        }

        WindowAttributes::default()
        .with_title(self.window_title)
        .with_inner_size(LogicalSize::new(self.window_width, self.window_height))
        .with_resizable(self.window_resizable)
        .with_visible(false)
        .with_enabled_buttons(window_enabled_buttons)
    }
}


pub trait ApplicationHandler {
    fn init(context: GraphicsContextRef<'static>, assets_manager: AssetsManagerRef) -> Self;
    fn update(&mut self, dt: f32) -> ApplicationSignal;
    fn draw(&mut self) -> Result<(), wgpu::SurfaceError>;
    fn handle_event(&mut self, event: ApplicationEvent) -> ApplicationSignal;
    fn handle_input(&mut self, input: &Input) -> ApplicationSignal;
}


pub struct Application<'a, Handler: ApplicationHandler> {
    handler: Option<Handler>,
    data: Option<AppData>,
    input: Input,
    settings: ApplicationSettings<'a>,

    timer: Timer,
}

impl<'a, Handler: ApplicationHandler> Application<'a, Handler> {

    pub fn new(settings: ApplicationSettings<'a>) -> Self {
        Self {
            handler: None,
            data: None,
            input: Input::new(),
            settings,
            
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


impl<'a, Handler: ApplicationHandler> winit::application::ApplicationHandler<AppData> for Application<'a, Handler> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        log::info!("Initializing application data and handler");
        
        let window = event_loop.create_window(self.settings.create_window_attributes()).unwrap();
        let data = smol::block_on(AppData::new(window));
        data.window.set_visible(true);

        self.handler = Some(Handler::init(data.context.clone(), data.assets_manager.clone()));

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

        let window_title = self.settings.window_title.to_string();
        data.window.set_title(&(window_title + &format!(" [FPS: {}]", 1.0 / elapsed_as_secs)));


        self.input.keyboard_input.set_released_keys_to_idle();
        
        let signal = match event {
            WindowEvent::CloseRequested => {event_loop.exit(); None}
            WindowEvent::RedrawRequested => {

                let signal = handler.update(elapsed_as_secs);

                match handler.draw() {
                    Ok(()) => (),
                    Err(wgpu::SurfaceError::Outdated | wgpu::SurfaceError::Lost) => {
                        let mut context = data.context.write().unwrap();
                        let width = context.config.width;
                        let height = context.config.height;
                        context.resize_surface(width, height);
                    }

                    Err(e) => log::error!("Error while drawing to surface {e:?}"),
                }

                data.window.request_redraw();

                Some(signal)
            }

            WindowEvent::Resized(size) => {
                let mut context = data.context.write().unwrap();
                context.resize_surface(size.width, size.height);

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
    context: GraphicsContextRef<'static>,
    assets_manager: AssetsManagerRef
}

impl AppData {
    async fn new(window: Window) -> Self {
        log::info!("init app data");
        let window = Arc::new(window);

        let size = window.inner_size();

        let context = Arc::new(
            RwLock::new(
            GraphicsContext
            ::new(
                window.clone(),
                 size.width,
                  size.height
            ).await));

        let mut assets_manager = AssetsManager::new();
        Self::register_assets_types(&mut assets_manager);
        Self::register_assets_loaders(&mut assets_manager, context.clone());
        Self {
            window,
            context,
            assets_manager: Arc::new(RwLock::new(assets_manager))
        }
    }

    fn register_assets_types(assets_manager: &mut AssetsManager) {
        assets_manager.register_assets_type::<Texture2D>().unwrap();
    }

    fn register_assets_loaders(assets_manager: &mut AssetsManager, context: GraphicsContextRef<'static>) {
        assets_manager.register_loader::<_, _, &str>(Texture2DLoader::new(context.clone()));
        assets_manager.register_loader::<_, _, RawRgbaImageData>(Texture2DLoader::new(context.clone()));
    }
}