use std::cell::Cell;


pub struct Quad {
    position: glam::Vec2,
    size: glam::Vec2,
    rotation: f32,
    pub z_index: i32,
    pub color: glam::Vec4,

    transform: Cell<glam::Mat4>,
    transform_needs_update: Cell<bool>,
}


impl Quad {

    pub fn with_position_and_size(position: glam::Vec2, size: glam::Vec2) -> Self {
        Self::new(position, size, 0.0)
    }

    pub fn with_size(size: glam::Vec2) -> Self {
        Self::with_position_and_size(glam::Vec2::ZERO, size)
    }

    pub fn set_position(&mut self, position: glam::Vec2) {
        self.position = position;
        self.transform_needs_update.set(true);
    }

    pub fn get_position(&self) -> glam::Vec2 {
        self.position
    }

    pub fn set_size(&mut self, size: glam::Vec2) {
        self.size = size;
        self.transform_needs_update.set(true);
    }

    pub fn get_size(&self) -> glam::Vec2 {
        self.size
    }

    pub fn rotate(&mut self, delta_rotation: f32) {
        self.rotation += delta_rotation;
        self.transform_needs_update.set(true);
    }

    pub fn set_rotation(&mut self, rotation: f32) {
        self.rotation = rotation;
        self.transform_needs_update.set(true);
    }

    pub fn get_rotation(&self) -> f32 {
        self.rotation
    }

    pub fn get_transform(&self) -> glam::Mat4 {
        if self.transform_needs_update.get() {
            self.transform_needs_update.set(false);
            self.transform.set(Self::compute_transform(self.position, self.size, self.rotation))
        }
        self.transform.get()
    }

    fn new(position: glam::Vec2, size: glam::Vec2, rotation: f32) -> Self {
        let transform = Self::compute_transform(position, size, rotation);

        Self {
            position,
            size,
            rotation,
            transform: Cell::new(transform),
            color: glam::vec4(1.0, 1.0, 1.0, 1.0),
            transform_needs_update: Cell::new(false),
            z_index: 0,
        }
    }


    fn compute_transform(position: glam::Vec2, size: glam::Vec2, rotation: f32) -> glam::Mat4 {
        let rotation_quat = glam::Quat::from_rotation_z(rotation.to_radians());

        let center = (size * 0.5).extend(0.0);
        let rotated_center = rotation_quat * -center;
        let final_translation = position.extend(0.0) + center + rotated_center;


        glam::Mat4::from_scale_rotation_translation(size.extend(1.0), rotation_quat, final_translation)
    }
}