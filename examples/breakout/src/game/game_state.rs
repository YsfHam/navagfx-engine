use std::f32;

use navagfx_engine::{application::input::{Input, KeyboardKey}, assets::{texture::Texture2D, AssetHandle, AssetsManagerRef}, export::{application_export::KeyCode, glam}, graphics::{renderer2d::Renderer2D, shapes::Quad}};

use navagfx_engine::{application::event::{ApplicationEvent, ApplicationSignal}, export::{graphics_export::Color}, graphics::camera::Camera2D};

use crate::{game::entities::{Ball, BrickType, BricksManager, Paddle}, physics::{circle_rectangle_collision_check, HitInfo}};


pub struct LevelData {
    pub bricks_rows: usize,
    pub bricks_cols: usize,
    pub bricks_types: Vec<BrickType>,
}

impl LevelData {
    pub fn load_from_file(file_path: &str) -> Self {
        // Read entire file contents as a String
        let data = std::fs::read_to_string(file_path).expect("Failed to read level file");
        let mut lines = data.lines();
        let mut meta_data = lines.next().unwrap().split_whitespace();

        let bricks_cols = meta_data.next().unwrap().parse().unwrap();
        let bricks_rows = meta_data.next().unwrap().parse().unwrap();

        let bricks_types = 
            lines
                .flat_map(|line| line.split_whitespace())
                .map(|brick_type_str| brick_type_str.parse::<u32>().unwrap())
                .map(BrickType::from)
                .collect::<Vec<_>>()
            ;

        // Sanity check: ensure data dimensions match the declared grid
        assert_eq!(
            bricks_types.len(),
            bricks_rows * bricks_cols,
            "Level bricks count mismatch: expected {} rows x {} cols = {}, got {}",
            bricks_rows,
            bricks_cols,
            bricks_rows * bricks_cols,
            bricks_types.len()
        );
        Self {
            bricks_rows,
            bricks_cols,
            bricks_types
        }    
    }
}


const PLAYER_VELOCITY: f32 = 400.0;
const BALL_VELOCITY: glam::Vec2 = glam::vec2(100.0, -300.0);
const BALL_RADIUS: f32 = 15.0;
const PADDLE_SIZE: glam::Vec2 = glam::vec2(128.0, 16.0);


fn get_center_over_rect(rect_pos: glam::Vec2, rect_size: glam::Vec2) -> glam::Vec2 {
    let half_size = rect_size * 0.5;
    let center_pos = rect_pos + half_size;

    glam::vec2(center_pos.x, center_pos.y - half_size.y)
}


pub struct GameState {
    camera: Camera2D,

    ball: Ball,
    paddle: Paddle,
    ball_idle: bool,

    bricks_mgr: BricksManager,
    window_width: f32,
    window_height: f32,

    background_texture: AssetHandle<Texture2D>,
}

impl GameState {
    pub fn new(window_width: f32, window_height: f32, assets_manager: AssetsManagerRef) -> Self {

        let level_data = LevelData::load_from_file("assets/levels/one.lvl");

        let mut assets_manager = assets_manager.lock().unwrap();
        let ball_texture = assets_manager.load_asset::<Texture2D, _>("assets/textures/awesomeface.png").unwrap();
        let background_texture = assets_manager.load_asset::<Texture2D, _>("assets/textures/background.jpg").unwrap();

        let solid_brick_texture = assets_manager.load_asset::<Texture2D, _>("assets/textures/block_solid.png").unwrap();
        let brick_texture = assets_manager.load_asset::<Texture2D, _>("assets/textures/block.png").unwrap();

        let paddle_texture = assets_manager.load_asset::<Texture2D, _>("assets/textures/paddle.png").unwrap();


        let paddle_pos = glam::vec2(
            (window_width - PADDLE_SIZE.x) * 0.5,
            window_height - PADDLE_SIZE.y
        );

        
        let paddle_surface_center = get_center_over_rect(paddle_pos, PADDLE_SIZE);
        let ball_position = glam::vec2(paddle_surface_center.x, paddle_surface_center.y - BALL_RADIUS);
        Self {
            camera: Camera2D::new(window_width, window_height),
            ball: Ball::new(ball_position, BALL_VELOCITY, BALL_RADIUS, ball_texture),
            paddle: Paddle::new(paddle_pos, PLAYER_VELOCITY, PADDLE_SIZE, paddle_texture),
            ball_idle: true,
            bricks_mgr: BricksManager::new(level_data, window_width, window_height * 0.5, solid_brick_texture, brick_texture),
            window_height,
            window_width,
            background_texture
        }
    }

    pub fn update(&mut self, dt: f32) -> ApplicationSignal {

        self.paddle.transform.update(dt);
        self.keep_paddle_inside_screen();

        if self.ball_idle {
            let paddle_pos = self.paddle.transform.position;
            let paddle_size = self.paddle.size;
            let paddle_surface_center = get_center_over_rect(paddle_pos, paddle_size);
            let ball_pos = glam::vec2(paddle_surface_center.x, paddle_surface_center.y - self.ball.radius);
            self.ball.transform.position = ball_pos;
        }
        else {
            self.ball.transform.update(dt);
            self.resolve_ball_collision();
            self.keep_ball_inside_screen();
        }


        if self.ball.transform.position.y - self.ball.radius > self.window_height {
            return ApplicationSignal::Exit;
        }

        ApplicationSignal::Continue
    }

    pub fn draw(&mut self, renderer: &mut Renderer2D) {

        renderer.begin(Color::BLACK, &self.camera);
        
        self.draw_background(renderer);

        self.ball.render(renderer);
        
        self.bricks_mgr.draw(renderer);
        
        self.paddle.render(renderer);
    }

    pub fn handle_event(&mut self, event: ApplicationEvent) -> ApplicationSignal {

        match event {
            ApplicationEvent::Resized { width, height } 
                => self.camera = Camera2D::new(width as f32, height as f32),
        }
        
        ApplicationSignal::Continue
    }
    
    pub fn handle_input(&mut self, input: &Input) -> ApplicationSignal {
        self.paddle.transform.velocity.x = 
        if input.keyboard_input.is_key_pressed(KeyboardKey::Code(KeyCode::ArrowLeft)) {
            -PLAYER_VELOCITY
        }
        else if input.keyboard_input.is_key_pressed(KeyboardKey::Code(KeyCode::ArrowRight)) {
            PLAYER_VELOCITY
        }
        else {
            0.0
        };

        if input.keyboard_input.is_key_pressed(KeyboardKey::Code(KeyCode::Space)) && self.ball_idle{
            self.ball_idle = false;
            self.ball.transform.velocity = BALL_VELOCITY;
        }

        ApplicationSignal::Continue
    }

    fn draw_background(&self, renderer: &mut Renderer2D) {
        let mut quad = Quad::with_position_and_size(
            glam::vec2(0.0, 0.0),
            glam::vec2(self.window_width, self.window_height),
        );

        quad.z_index = -100;

        renderer.draw_quad_textured( &quad, self.background_texture, Default::default());
    }

    fn keep_paddle_inside_screen(&mut self) {

        let x_pos = self.paddle.transform.position.x;
        self.paddle.transform.position.x = x_pos.clamp(0.0, self.window_width - self.paddle.size.x);
    }

    fn keep_ball_inside_screen(&mut self) {

        let ball_pos = self.ball.transform.position;
        let ball_radius = self.ball.radius;

        if ball_pos.x < ball_radius || ball_pos.x + ball_radius > self.window_width {
            self.ball.transform.velocity.x *= -1.0;
        }

        if ball_pos.y < ball_radius {
            self.ball.transform.velocity.y *= -1.0;
        }
    }

    fn check_ball_paddle_collision(&self) -> Option<HitInfo> {
        let circle = self.ball.get_collider();

        let paddle_rect = self.paddle.get_collider();

        circle_rectangle_collision_check(&circle, &paddle_rect)
    }

    fn resolve_ball_collision(&mut self) {

        self.resolve_bricks_collisions();
        self.resolve_paddle_collisions();
    }

    fn resolve_bricks_collisions(&mut self) {
        let hit_infos = self.bricks_mgr.check_collisions(&self.ball);

        let velocity = self.ball.transform.velocity;
        
        let (new_vel_opt, pos_offset) = hit_infos.iter()
        .fold((None, glam::Vec2::ZERO), |(mut vel_acc, mut pos_acc), hit_info| {

            let normal = hit_info.hit_side_normal;
            let reflection_vel = velocity - 2.0 * velocity.dot(normal) * normal;

            let vel = vel_acc.get_or_insert(glam::Vec2::ZERO);
            *vel += reflection_vel;

            let penetration_length = self.ball.radius - hit_info.circle_to_hit_point.length();
            pos_acc += normal * penetration_length;

            (vel_acc, pos_acc)
        });

        let new_vel = new_vel_opt.unwrap_or(velocity);
        self.ball.transform.velocity = velocity.length() * new_vel.normalize();
        self.ball.transform.position += pos_offset;
    }

    fn resolve_paddle_collisions(&mut self) {
        let hit_info_opt = self.check_ball_paddle_collision();
        if hit_info_opt.is_none() {
            return;
        }

        let half_size = self.paddle.size * 0.5;
        let paddle_center = self.paddle.transform.position + half_size;
        let dist_to_center = self.ball.transform.position.x + self.ball.radius - paddle_center.x;
        let percentage = dist_to_center / half_size.x;
        let strength = 2.0;
        let new_ball_vel_x = BALL_VELOCITY.x * percentage * strength;

        let old_ball_vel = self.ball.transform.velocity;
        self.ball.transform.velocity.x = new_ball_vel_x;
        self.ball.transform.velocity.y *= -1.0;

        self.ball.transform.velocity = old_ball_vel.length() * self.ball.transform.velocity.normalize();

        let hit_info = hit_info_opt.unwrap();

        let penetration = self.ball.radius - hit_info.hit_side_normal.length();
        self.ball.transform.position.y -= penetration;
    }

}
