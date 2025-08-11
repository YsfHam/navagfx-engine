use std::{slice::Iter, sync::{Arc, Mutex}, time::Duration};

use navagfx_engine::{application::{event::{ApplicationEvent, ApplicationSignal, KeyInfo}, Application, ApplicationHandler}, assets::{texture::Texture2D, AssetHandle, AssetsManager}, export::{application_export::KeyCode, glam, graphics_export::Color, image}, graphics::{camera::Camera2D, renderer2d::{AtlasTextureCoordinates, Renderer2D}, shapes::Quad, GraphicsContext}};


fn load_static_texture(context: &GraphicsContext, path: &str) -> Texture2D {
    let image = image::ImageReader::open(path).unwrap().decode().unwrap().to_rgba8();

    Texture2D::from_image(context, path, &image)
}


struct AtlasTexture {
    atlas_coords: Vec<AtlasTextureCoordinates>,
    rows: usize,
}

impl AtlasTexture {
    fn new(texture: &Texture2D, sprite_size: (u32, u32)) -> Self {
        let (sprite_width, sprite_height) = sprite_size;

        let size = [
            sprite_width as f32 / texture.width as f32,
            sprite_height as f32 / texture.height as f32,
        ];

        let mut atlas_coords = vec![];

        for y in 0.. texture.height / sprite_height {
            for x in 0.. texture.width / sprite_width {
                let offset = [
                    (x * sprite_width) as f32 / texture.width as f32,
                    (y * sprite_height) as f32 / texture.height as f32
                ];

                atlas_coords.push(AtlasTextureCoordinates {
                    tex_coords_offset: offset,
                    tex_coords_size: size
                });

                log::info!("atlas at {x} {y} size: {size:?}, offset {offset:?}");
            }
        }

        //panic!("Dont panic it is just me");

        Self {
            atlas_coords,
            rows: size[0] as usize
        }
    }

    fn get_coords(&self, x: usize, y: usize) -> Option<AtlasTextureCoordinates> {
        self.atlas_coords.get(y * self.rows + x).copied()
    }

    fn get_coords_by_index(&self, index: usize) -> Option<AtlasTextureCoordinates> {
        self.atlas_coords.get(index).copied()
    }
}

struct Animation {
    atlas_tex: AtlasTexture,
    frames_index_iter: std::iter::Cycle<std::ops::Range<usize>>,
    current_frame: usize,
    frame_time: Duration,
    frame_timer: Option<std::time::Instant>,
}

impl Animation {
    fn new(atlas_tex: AtlasTexture, frame_time: Duration) -> Self {
        let atlases_count = atlas_tex.atlas_coords.len();
        Self {
            atlas_tex,
            frames_index_iter: (0..atlases_count).cycle(),
            current_frame: 0,
            frame_time,
            frame_timer: None,
        }
    }

    fn get_frame_coords(&mut self) -> AtlasTextureCoordinates {
        let timer = self.frame_timer.get_or_insert_with(|| std::time::Instant::now());

        if timer.elapsed() > self.frame_time {
            self.current_frame = self.frames_index_iter.next().unwrap();
            *timer = std::time::Instant::now(); 
        }

        self.atlas_tex.get_coords_by_index(self.current_frame).unwrap()
    }
}

struct MyAppHandler {
    renderer2d: Renderer2D,
    current_angle: f32,

    happy_face_tex: AssetHandle<Texture2D>,
    happy_tree_tex: AssetHandle<Texture2D>,
    samurai_idle_tex: AssetHandle<Texture2D>,
    samurai_idle_animation: Animation,
    quads: Vec<Quad>
}


impl ApplicationHandler for MyAppHandler {
    fn init(context: &GraphicsContext, assets_manager: Arc<Mutex<AssetsManager>>) -> Self {
        log::info!("Application is initialised");

        let renderer2d = Renderer2D::new(context, assets_manager.clone());

        let mut lock = assets_manager.lock().unwrap();

        let happy_face_tex = lock.store_asset(load_static_texture(context, "./assets/happy-face.jpg"));
        let happy_tree_tex = lock.store_asset(load_static_texture(context, "./assets/happy-tree.png"));

        let tex = load_static_texture(context, "./assets/IDLE.png");
        let samurai_idle_tex_atlas = AtlasTexture::new(&tex, (96, 96));
        let samurai_tex = lock.store_asset(tex);


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

                //quads.push(quad);
            }
        }

        Self {
            renderer2d,
            current_angle: 0.0,
            happy_face_tex,
            happy_tree_tex,
            samurai_idle_tex: samurai_tex,
            samurai_idle_animation: Animation::new(samurai_idle_tex_atlas, Duration::from_millis(16 * 8)),
            quads
        }
    }

    fn update(&mut self) -> ApplicationSignal {

        ApplicationSignal::Continue
    }

    fn draw(&mut self, context: &GraphicsContext) {

        let width = context.config.width as f32;
        let height = context.config.height as f32;

        self.renderer2d.begin(Color{r: 0.01, g:0.01, b:0.01, a:1.0}, &Camera2D::new(width, height));

        for quad in &self.quads {
            self.renderer2d.draw_quad_textured(&quad, self.samurai_idle_tex, self.samurai_idle_animation.get_frame_coords());
        }

        self.renderer2d.submit(context).unwrap();
    }

    fn handle_event(&mut self, event: ApplicationEvent) -> ApplicationSignal {
        log::info!("Handling event {event:?}");

        if let ApplicationEvent::KeyPressed { key_info: KeyInfo {physical_key_code: KeyCode::Escape, ..}, .. } = event {
            return ApplicationSignal::Exit;
        }

        if let ApplicationEvent::KeyPressed { key_info: KeyInfo {physical_key_code: KeyCode::KeyS, ..}, .. } = event {
            self.quads.push(Quad::new(
                glam::vec2(32.0, 100.0),
                glam::vec2(200.0, 200.0),
                0.0
            ));
        }

        ApplicationSignal::Continue
    }
}

fn main() {
    simple_logger::init_with_level(log::Level::Debug).unwrap();

    let app = Application::<MyAppHandler>::new();
    app.run();
}
