use navagfx_engine::{application::input::{Input, KeyboardKey}, export::{application_export::KeyCode, glam}, graphics::{renderer2d::Renderer2D, shapes::Quad}};

use navagfx_engine::{application::{event::{ApplicationEvent, ApplicationSignal}, ApplicationHandler}, assets::AssetsManagerRef, export::{graphics_export::{Color, SurfaceError}}, graphics::{camera::Camera2D, GraphicsContext}};


struct DynamicObject {
    velocity: f32,
    direction: glam::Vec2,
    quad: Quad,
}

impl DynamicObject {
    fn new(quad: Quad, velocity: f32) -> Self {
        Self {
            quad,
            velocity,
            direction: glam::Vec2::ZERO
        }
    }

    fn set_direction(&mut self, direction: glam::Vec2) {
        self.direction = direction;
    }

    fn update(&mut self, dt: f32) {
        let current_pos = self.quad.get_position();

        let new_pos = current_pos + self.velocity * self.direction * dt;
        self.quad.set_position(new_pos);
    }

    fn render(&self, renderer: &mut Renderer2D) {
        renderer.draw_quad(&self.quad);
    }
}

struct Ball {
    object: DynamicObject,
    radius: f32,
}

impl Ball {

    fn new(position: glam::Vec2, radius: f32) -> Self {

        let quad_side_length = radius * 2.0;

        let mut ball_quad = Quad::new(position, glam::vec2(quad_side_length, quad_side_length), 0.0);
        ball_quad.color = glam::vec4(1.0, 0.0, 0.0, 1.0);
        Self {
            object: DynamicObject::new(ball_quad, 100.0),
            radius
        }
    }
}


pub struct GameApp {
    renderer: Renderer2D,
    camera: Camera2D,

    ball: Ball,
    launch_base: DynamicObject,
    ball_idle: bool,
}

impl GameApp {    
}

impl ApplicationHandler for GameApp {
    fn init(context: &GraphicsContext, assets_manager: AssetsManagerRef) -> Self {
        let renderer = Renderer2D::new(context, assets_manager);

        let width = context.config.width as f32;
        let height = context.config.height as f32;

        let launch_base_size = glam::vec2(100.0, 20.0);
        let launch_base_pos = glam::vec2(
            (width - launch_base_size.x) * 0.5,
            height - launch_base_size.y
        );

        let mut launch_base_quad = Quad::new(launch_base_pos, launch_base_size, 0.0);
        launch_base_quad.color = glam::vec4(0.0, 1.0, 0.0, 1.0);

        let ball_radius = 30.0;
        let ball_position = glam::vec2(
            launch_base_pos.x + launch_base_size.x * 0.5 - ball_radius,
            launch_base_pos.y - ball_radius * 2.0
        );

        Self {
            renderer,
            camera: Camera2D::new(width, height),
            ball: Ball::new(ball_position, ball_radius),
            launch_base: DynamicObject::new(launch_base_quad, 200.0),
            ball_idle: true,
        }
    }

    fn update(&mut self, dt: f32) -> ApplicationSignal {

        self.launch_base.update(dt);
        if self.ball_idle {
            let pos = self.launch_base.quad.get_position();
            let size = self.launch_base.quad.get_size();
            self.ball.object.quad.set_position(glam::vec2(
                pos.x + size.x * 0.5 - self.ball.radius,
                pos.y - self.ball.radius * 2.0
            ));
        }
        else {
            self.ball.object.update(dt);
        }

        ApplicationSignal::Continue
    }

    fn draw(&mut self, context: &GraphicsContext) -> Result<(), SurfaceError> {

        self.renderer.begin(Color::BLACK, &self.camera);
        self.launch_base.render(&mut self.renderer);
        self.ball.object.render(&mut self.renderer);
        self.renderer.submit(context)
    }

    fn handle_event(&mut self, event: ApplicationEvent) -> ApplicationSignal {

        match event {
            ApplicationEvent::Resized { width, height } 
                => self.camera = Camera2D::new(width as f32, height as f32),
        }
        
        ApplicationSignal::Continue
    }
    
    fn handle_input(&mut self, input: &Input) -> ApplicationSignal {
        if input.keyboard_input.is_key_pressed(KeyboardKey::Code(KeyCode::ArrowLeft)) {
            self.launch_base.set_direction(glam::vec2(-1.0, 0.0));
        }
        else if input.keyboard_input.is_key_pressed(KeyboardKey::Code(KeyCode::ArrowRight)) {
            self.launch_base.set_direction(glam::vec2(1.0, 0.0));
        }
        else {
            self.launch_base.set_direction(glam::vec2(0.0, 0.0));
        }

        if input.keyboard_input.is_key_pressed(KeyboardKey::Code(KeyCode::Space)) && self.ball_idle{
            self.ball_idle = false;
            self.ball.object.set_direction(glam::vec2(0.0, -1.0));
        }


        if input.keyboard_input.is_key_released(KeyboardKey::Code(KeyCode::Escape)) {
            return ApplicationSignal::Exit;
        }

        ApplicationSignal::Continue
    }
}
