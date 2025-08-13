use navagfx_engine::{application::input::{Input, KeyboardKey}, export::{application_export::KeyCode, glam}, graphics::{renderer2d::Renderer2D, shapes::Quad}};

use navagfx_engine::{application::event::{ApplicationEvent, ApplicationSignal}, export::{graphics_export::Color}, graphics::camera::Camera2D};

use crate::physics::{circle_rectangle_collision_check, Circle, Rectangle};


enum BrickType {
    None,
    Solid,
    Destroyable(u32)
}

impl From<u32> for BrickType {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::None,
            1 => Self::Solid,
            n => Self::Destroyable(n)
        }
    }
}

pub struct LevelData {
    bricks_rows: usize,
    bricks_cols: usize,
    bricks_types: Vec<BrickType>,
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


struct Brick {
    quad: Quad,
    is_solid: bool,
    destroyed: bool,
}

struct BricksManager {
    bricks: Vec<Brick>
}

impl BricksManager {
    fn new(level_data: LevelData, lvl_width: f32, lvl_height: f32) -> Self {
        let brick_width = lvl_width / level_data.bricks_cols as f32;
        let brick_height = lvl_height / level_data.bricks_rows as f32;

        let mut bricks = Vec::with_capacity(level_data.bricks_types.len());

        for y in 0..level_data.bricks_rows {
            for x in 0..level_data.bricks_cols {
                let brick_type = level_data.bricks_types.get(y * level_data.bricks_cols + x).unwrap();
                let (color, is_solid) = match brick_type {
                    BrickType::None => continue,
                    BrickType::Solid => (glam::vec4(0.5, 0.5, 0.5, 1.0), true),
                    BrickType::Destroyable(id) => (Self::get_brick_color(*id), false),
                };

                let pos = glam::vec2(x as f32 * brick_width, y as f32 * brick_height);
                let size = glam::vec2(brick_width, brick_height);
                let mut quad = Quad::new(pos, size, 0.0);
                quad.color = color;
                bricks.push(Brick {
                    quad,
                    is_solid,
                    destroyed: false,
                })
            }
        }

        Self {
            bricks
        }
    }

    fn get_brick_color(id: u32) -> glam::Vec4 {
        match id {
            2 => glam::vec4(0.2, 0.6, 1.0, 1.0),
            3 => glam::vec4(0.0, 0.7, 0.0, 1.0),
            4 => glam::vec4(0.8, 0.8, 0.4, 1.0),
            5 => glam::vec4(1.0, 0.5, 0.0, 1.0),
            _ => panic!("Unknow brick id {id}")
        }
    }

    fn draw(&self, renderer: &mut Renderer2D) {

        self.bricks.iter()
        .filter(|brick| !brick.destroyed)
        .for_each(|brick| renderer.draw_quad(&brick.quad));
    }

    fn check_collisions(&mut self, ball: &Ball) -> bool {
        let circle = Circle {
            radius: ball.radius,
            position: ball.object.quad.get_position()
        };
        self.bricks.iter_mut()
        .filter(|brick| !brick.destroyed)
        .map(|brick| {
            let half_size = brick.quad.get_size() * 0.5;
            let rect = Rectangle {
                position: brick.quad.get_position() + half_size,
                size: half_size
            };

            (brick, rect)
        })
        .fold(false, |acc, (brick, rect)| {
            let collided = circle_rectangle_collision_check(&circle, &rect);
            brick.destroyed = collided && !brick.is_solid;

            collided || acc
        })
    }

}


struct DynamicObject {
    velocity: glam::Vec2,
    direction: glam::Vec2,
    quad: Quad,
}

impl DynamicObject {
    fn new(quad: Quad, velocity: glam::Vec2) -> Self {
        Self {
            quad,
            velocity,
            direction: glam::Vec2::ZERO
        }
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

    fn new(position: glam::Vec2, radius: f32, velocity: glam::Vec2) -> Self {

        let quad_side_length = radius * 2.0;

        let mut ball_quad = Quad::new(position, glam::vec2(quad_side_length, quad_side_length), 0.0);
        ball_quad.color = glam::vec4(1.0, 0.0, 0.0, 1.0);
        Self {
            object: DynamicObject::new(ball_quad, velocity),
            radius
        }
    }
}

const PLAYER_VELOCITY: f32 = 400.0;
const BALL_VELOCITY: glam::Vec2 = glam::vec2(100.0, 300.0);
const BALL_RADIUS: f32 = 10.0;
const PADDLE_SIZE: glam::Vec2 = glam::vec2(100.0, 10.0);


pub struct GameState {
    camera: Camera2D,

    ball: Ball,
    paddle: DynamicObject,
    ball_idle: bool,

    bricks_mgr: BricksManager,
    window_width: f32,
    window_height: f32
}

impl GameState {
    pub fn new(window_width: f32, window_height: f32, level_data: LevelData) -> Self {

        let paddle_size = PADDLE_SIZE;
        let paddle_pos = glam::vec2(
            (window_width - paddle_size.x) * 0.5,
            window_height - paddle_size.y
        );

        let mut paddle_quad = Quad::new(paddle_pos, paddle_size, 0.0);
        paddle_quad.color = glam::vec4(0.0, 1.0, 0.0, 1.0);

        let ball_radius = BALL_RADIUS;
        let ball_position = glam::vec2(
            paddle_pos.x + paddle_size.x * 0.5 - ball_radius,
            paddle_pos.y - ball_radius * 2.0
        );

        let player_vel = glam::vec2(PLAYER_VELOCITY, 0.0);
        Self {
            camera: Camera2D::new(window_width, window_height),
            ball: Ball::new(ball_position, ball_radius, player_vel),
            paddle: DynamicObject::new(paddle_quad, player_vel),
            ball_idle: true,
            bricks_mgr: BricksManager::new(level_data, window_width, window_height * 0.5),
            window_height,
            window_width
        }
    }

    pub fn update(&mut self, dt: f32) -> ApplicationSignal {

        
        if self.ball_idle {
            self.ball.object.direction = self.paddle.direction;
        }

        self.paddle.update(dt);
        self.ball.object.update(dt);
        if self.bricks_mgr.check_collisions(&self.ball) {
            self.ball.object.direction *= -1.0;
        }

        self.keep_ball_inside_screen();

        if self.check_ball_paddle_collision() {
            self.ball.object.direction.y = -1.0;
        }
        

        ApplicationSignal::Continue
    }

    pub fn draw(&mut self, renderer: &mut Renderer2D) {

        renderer.begin(Color::BLACK, &self.camera);

        self.bricks_mgr.draw(renderer);
        
        self.paddle.render(renderer);
        self.ball.object.render(renderer);
    }

    pub fn handle_event(&mut self, event: ApplicationEvent) -> ApplicationSignal {

        match event {
            ApplicationEvent::Resized { width, height } 
                => self.camera = Camera2D::new(width as f32, height as f32),
        }
        
        ApplicationSignal::Continue
    }
    
    pub fn handle_input(&mut self, input: &Input) -> ApplicationSignal {
        if input.keyboard_input.is_key_pressed(KeyboardKey::Code(KeyCode::ArrowLeft)) {
            self.paddle.direction.x = -1.0;
        }
        else if input.keyboard_input.is_key_pressed(KeyboardKey::Code(KeyCode::ArrowRight)) {
            self.paddle.direction.x = 1.0;
        }
        else {
            self.paddle.direction.x = 0.0;
        }

        self.keep_paddle_inside_screen();

        if input.keyboard_input.is_key_pressed(KeyboardKey::Code(KeyCode::Space)) && self.ball_idle{
            self.ball_idle = false;
            self.ball.object.velocity = BALL_VELOCITY;
            self.ball.object.direction = glam::vec2(-1.0, -1.0);
        }

        ApplicationSignal::Continue
    }

    fn keep_paddle_inside_screen(&mut self) {
        let paddle_pos_x = self.paddle.quad.get_position().x;

        if paddle_pos_x < 0.0 {
            self.paddle.direction.x = 1.0;
        }
        else if paddle_pos_x + self.paddle.quad.get_size().x > self.window_width {
            self.paddle.direction.x = -1.0;
        }
    }

    fn keep_ball_inside_screen(&mut self) {
        if self.ball_idle {
            return;
        }

        let ball_pos = self.ball.object.quad.get_position();
        let ball_size = self.ball.object.quad.get_size();
        let mut ball_direction = self.ball.object.direction;


        if ball_pos.x < 0.0 {
            ball_direction.x = 1.0;
        }
        else if ball_pos.x + ball_size.x > self.window_width {
            ball_direction.x = -1.0;
        }

        if ball_pos.y < 0.0 {
            ball_direction.y = 1.0;
        }

        self.ball.object.direction = ball_direction;
    }

    fn check_ball_paddle_collision(&self) -> bool {
        let circle = Circle {
            radius: self.ball.radius,
            position: self.ball.object.quad.get_position()
        };

        let paddle_rect = Rectangle::from(&self.paddle.quad);

        circle_rectangle_collision_check(&circle, &paddle_rect)
    }

}
