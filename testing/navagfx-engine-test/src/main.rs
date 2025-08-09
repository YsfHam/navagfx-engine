use navagfx_engine::{application::{event::{ApplicationEvent, ApplicationSignal, KeyCode, KeyInfo}, Application, ApplicationHandler}, graphics::{renderer2d::Renderer2D, GraphicsContext}};

struct MyAppHandler {
    renderer2d: Option<Renderer2D>,
}

impl MyAppHandler {
    fn new() -> Self {
        Self {
            renderer2d: None
        }
    }
}


impl ApplicationHandler for MyAppHandler {
    fn init(&mut self, context: &GraphicsContext) {
        log::info!("Application is initialised");

        self.renderer2d = Some(Renderer2D::new(context));
    }

    fn update(&mut self) -> ApplicationSignal {

        ApplicationSignal::Continue
    }

    fn draw(&mut self, context: &GraphicsContext) {
        let renderer2d = self.renderer2d.as_ref().unwrap();

        renderer2d.submit(context).unwrap();
    }

    fn handle_event(&mut self, event: ApplicationEvent) -> ApplicationSignal {
        log::info!("Handling event {event:?}");

        if let ApplicationEvent::KeyPressed { key_info: KeyInfo {physical_key_code: KeyCode::Escape, ..}, .. } = event {
            return ApplicationSignal::Exit;
        }

        ApplicationSignal::Continue
    }
}

fn main() {
    simple_logger::init_with_level(log::Level::Debug).unwrap();

    let app = Application::new(MyAppHandler::new());
    app.run();
}
