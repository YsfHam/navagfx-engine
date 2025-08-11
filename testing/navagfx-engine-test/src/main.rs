use std::sync::{Arc, Mutex};

use navagfx_engine::{application::{event::{ApplicationEvent, ApplicationSignal, KeyInfo}, Application, ApplicationHandler}, assets::{texture::Texture2D, AssetHandle, AssetsManager}, export::{application_export::KeyCode, glam, graphics_export::Color, image}, graphics::{camera::Camera2D, renderer2d::Renderer2D, shapes::Quad, GraphicsContext}};


fn load_static_texture(context: &GraphicsContext, path: &str) -> Texture2D {
    let image = image::ImageReader::open(path).unwrap().decode().unwrap().to_rgba8();

    Texture2D::from_image(context, path, &image)
}

struct MyAppHandler {
    renderer2d: Renderer2D,
    current_angle: f32,

    happy_face_tex: AssetHandle<Texture2D>,
    happy_tree_tex: AssetHandle<Texture2D>,

    quads: Vec<Quad>
}


impl ApplicationHandler for MyAppHandler {
    fn init(context: &GraphicsContext, assets_manager: Arc<Mutex<AssetsManager>>) -> Self {
        log::info!("Application is initialised");

        let renderer2d = Renderer2D::new(context, assets_manager.clone());

        let mut lock = assets_manager.lock().unwrap();

        let happy_face_tex = lock.store_asset(load_static_texture(context, "./assets/happy-face.jpg"));
        let happy_tree_tex = lock.store_asset(load_static_texture(context, "./assets/happy-tree.png"));


        let mut quads = vec![];

        let spacing = glam::vec2(5.0, 5.0);
        let init_pos = glam::vec2(10.0, 10.0);
        for y in 0..100 {
            for x in 0..100 {
                let size = glam::vec2(30.0, 30.0);
                let pos = 
                    glam::vec2(x as f32 * (size.x + spacing.x), y as f32 * (size.y + spacing.y))
                    + init_pos;
                
                let quad = Quad::new(
                    pos,
                    size,
                    0.0
                );

                //quad.color = glam::vec4((x as f32).cos() * 0.5 + 0.5, (y as f32).cos() * 0.5 + 0.5, 0.4, 1.0);

                quads.push(quad);
            }
        }

        Self {
            renderer2d,
            current_angle: 0.0,
            happy_face_tex,
            happy_tree_tex,
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

        self.renderer2d.begin(Color{r: 0.01, g:0.01, b:0.01, a:1.0}, &Camera2D::new(width, height));

        let mut face = true;

        for quad in &self.quads {
            let tex = if face {self.happy_face_tex} else {self.happy_tree_tex};
            self.renderer2d.draw_quad_textured(quad, tex);
            face = !face;
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
