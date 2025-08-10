use navagfx_engine::{application::{event::{ApplicationEvent, ApplicationSignal, KeyInfo}, Application, ApplicationHandler}, export::{application_export::KeyCode, glam, graphics_export::Color}, graphics::{camera::Camera2D, renderer2d::Renderer2D, shapes::Quad, GraphicsContext}};

struct MyAppHandler {
    renderer2d: Renderer2D,
    current_angle: f32,
    quads: Vec<Quad>
}


impl ApplicationHandler for MyAppHandler {
    fn init(context: &GraphicsContext) -> Self {
        log::info!("Application is initialised");

        let renderer2d = Renderer2D::new(context);

        let mut quads = vec![];

        let spacing = glam::vec2(5.0, 5.0);
        let init_pos = glam::vec2(10.0, 10.0);
        for y in 0..100 {
            for x in 0..1000 {
                let size = glam::vec2(10.0, 10.0);
                let pos = 
                    glam::vec2(x as f32 * (size.x + spacing.x), y as f32 * (size.y + spacing.y))
                    + init_pos;
                
                let mut quad = Quad::new(
                    pos,
                    size,
                    0.0
                );

                quad.color = glam::vec4((x as f32).cos() * 0.5 + 0.5, (y as f32).cos() * 0.5 + 0.5, 0.4, 1.0);

                quads.push(quad);
            }
        }

        Self {
            renderer2d,
            current_angle: 0.0,
            quads
        }
    }

    fn update(&mut self) -> ApplicationSignal {

        ApplicationSignal::Continue
    }

    fn draw(&mut self, context: &GraphicsContext) {

        let width = context.config.width as f32;
        let height = context.config.height as f32;

        self.current_angle += 0.5;
        if self.current_angle > 360.0 {
            self.current_angle = 0.0;
        }

        self.renderer2d.begin(Color::GREEN, &Camera2D::new(width, height));

        for quad in &self.quads {
            self.renderer2d.draw_quad(quad);
        }
        self.renderer2d.submit(context).unwrap();
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

    let app = Application::<MyAppHandler>::new();
    app.run();
}
