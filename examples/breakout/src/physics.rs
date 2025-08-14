use navagfx_engine::export::glam;

pub struct Circle {
    pub radius: f32,
    pub position: glam::Vec2,
}

pub struct Rectangle {
    pub position: glam::Vec2,
    pub size: glam::Vec2,
}

pub struct HitInfo {
    //pub hit_point: glam::Vec2,
    pub hit_side_normal: glam::Vec2,
    pub circle_to_hit_point: glam::Vec2,
}

static DIRECTIONS_VECS: &[glam::Vec2] = &[
    glam::vec2(0.0, -1.0), // UP
    glam::vec2(0.0, 1.0), // DOWN
    glam::vec2(-1.0, 0.0), // LEFT
    glam::vec2(1.0, 0.0), // RIGHT
];

pub fn circle_rectangle_collision_check(circle: &Circle, rect: &Rectangle) -> Option<HitInfo> {
    let diff_vector = circle.position - rect.position;
    let clamped = diff_vector.clamp(-rect.size, rect.size);
    let hit_point = clamped + rect.position;

    let circle_to_hit_point = circle.position - hit_point;

    let collided = circle_to_hit_point.length_squared() < (circle.radius * circle.radius);
    
    if !collided {
        return None;
    }

    Some(HitInfo {
        hit_side_normal: get_hit_direction(circle_to_hit_point),
        circle_to_hit_point
    })
}

fn get_hit_direction(target: glam::Vec2) -> glam::Vec2 {
    let target_norm = target.normalize();

    DIRECTIONS_VECS.iter()
    .fold(None, |acc, dir| {
        let dir_dot = dir.dot(target_norm);
        if acc.is_some_and(|(_, acc_dot)|  acc_dot > dir_dot) {
            acc
        }
        else {
            Some((dir, dir_dot))
        }
    })
    .map(|(dir, _)| dir)
    .copied()
    .unwrap()
}