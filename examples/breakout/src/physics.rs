use navagfx_engine::{export::glam, graphics::shapes::Quad};

pub struct Circle {
    pub radius: f32,
    pub position: glam::Vec2,
}

pub struct Rectangle {
    pub position: glam::Vec2,
    pub size: glam::Vec2,
}

impl From<&Quad> for Rectangle {
    fn from(value: &Quad) -> Self {
        let half_size = value.get_size() * 0.5;
        Self {
            position: value.get_position() + half_size,
            size: half_size
        }
    }
}

pub fn circle_rectangle_collision_check(circle: &Circle, rect: &Rectangle) -> bool {
    let diff_vector = circle.position - rect.position;
    let clamped = diff_vector.clamp(-rect.size, rect.size);
    let closest = clamped + rect.position;

    (circle.position - closest).length_squared() < (circle.radius * circle.radius)
}