use navagfx_engine::{export::glam, graphics::{renderer2d::Renderer2D, shapes::Quad}};

use crate::{game::game_state::LevelData, physics::{circle_rectangle_collision_check, Circle, HitInfo, Rectangle}};


pub struct Transform {
    pub position: glam::Vec2,
    pub velocity: glam::Vec2,
}

impl Transform {
    pub fn update(&mut self, dt: f32) {
        self.position += self.velocity * dt;
    }
}

pub struct Ball {
    pub transform: Transform,
    pub radius: f32,
}

impl Ball {

    pub fn new(position: glam::Vec2, velocity: glam::Vec2, radius: f32) -> Self {
        Self{
            transform: Transform {
                position,
                velocity,
            },
            radius
        }
    }

    
    pub fn get_collider(&self) -> Circle {
        Circle {
            radius: self.radius,
            position: self.transform.position,
        }
    }

    pub fn render(&self, renderer: &mut Renderer2D) {

        let quad_half_size = glam::vec2(self.radius, self.radius);
        let quad_position = self.transform.position - quad_half_size;

        let mut quad = Quad::new(quad_position, quad_half_size * 2.0, 0.0);
        quad.color = glam::vec4(1.0, 0.0, 0.0, 1.0);
        renderer.draw_quad(&quad);
    }
}


pub struct Paddle {
    pub transform: Transform,
    pub size: glam::Vec2,
}

impl Paddle {

    pub fn new(position: glam::Vec2, velocity: f32, size: glam::Vec2) -> Self {
        Self{
            transform: Transform {
                position,
                velocity: glam::vec2(velocity, 0.0)
            },
            size
        }
    }

    
    pub fn get_collider(&self) -> Rectangle {
        let half_size = self.size * 0.5;
        Rectangle {
            position: self.transform.position + half_size,
            size: half_size
        }
    }

    pub fn render(&self, renderer: &mut Renderer2D) {

        let mut quad = Quad::new(self.transform.position, self.size, 0.0);
        quad.color = glam::vec4(0.0, 1.0, 0.0, 1.0);
        renderer.draw_quad(&quad);
    }
}


pub enum BrickType {
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

struct Brick {
    quad: Quad,
    is_solid: bool,
    destroyed: bool,
}

pub struct BricksManager {
    bricks: Vec<Brick>
}

impl BricksManager {
    pub fn new(level_data: LevelData, lvl_width: f32, lvl_height: f32) -> Self {
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

    pub fn draw(&self, renderer: &mut Renderer2D) {

        self.bricks.iter()
        .filter(|brick| !brick.destroyed)
        .for_each(|brick| renderer.draw_quad(&brick.quad));
    }

    pub fn check_collisions(&mut self, ball: &Ball) -> Vec<HitInfo> {
        let circle = ball.get_collider();
        self.bricks.iter_mut()
        .filter(|brick| !brick.destroyed)
        .filter_map(|brick| {
            let half_size = brick.quad.get_size() * 0.5;
            let rect = Rectangle {
                position: brick.quad.get_position() + half_size,
                size: half_size
            };

            let hit_info = circle_rectangle_collision_check(&circle, &rect);
            brick.destroyed = hit_info.is_some() && !brick.is_solid;

            hit_info
        })
        .collect()
    }

}

