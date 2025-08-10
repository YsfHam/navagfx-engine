
pub struct Quad {
    pub position: glam::Vec2,
    pub size: glam::Vec2,
    pub rotation: glam::Quat,
    pub color: glam::Vec4,

    transform: glam::Mat4,
}


impl Quad {

    pub fn new(position: glam::Vec2, size: glam::Vec2, rotation_deg: f32) -> Self {
        let rotation = glam::Quat::from_rotation_z(rotation_deg.to_radians());
        let transform = Self::compute_transform(position, size, rotation);

        Self {
            position,
            size,
            rotation,
            transform,
            color: glam::vec4(1.0, 1.0, 1.0, 1.0)
        }
    }

    pub fn get_transform(&self) -> glam::Mat4 {
        self.transform
    }


    fn compute_transform(position: glam::Vec2, size: glam::Vec2, rotation: glam::Quat) -> glam::Mat4 {
        // let translation = glam::Mat4::from_translation(position.extend(0.0));
        // let rotation_mat = glam::Mat4::from_quat(rotation);
        // let scale = glam::Mat4::from_scale(size.extend(1.0));

        // let center_pos = size * 0.5;
        // let center_tanslation = glam::Mat4::from_translation(-center_pos.extend(0.0));
        // let inverse_center_tanslation = glam::Mat4::from_translation(center_pos.extend(0.0));

        // translation * inverse_center_tanslation * rotation_mat * center_tanslation * scale


        let center = (size * 0.5).extend(0.0);
        let rotated_center = rotation * -center;
        let final_translation = position.extend(0.0) + center + rotated_center;


        glam::Mat4::from_scale_rotation_translation(size.extend(1.0), rotation, final_translation)
    }
}