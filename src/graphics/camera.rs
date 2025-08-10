pub struct Camera2D {
    view_proj: glam::Mat4,
}

impl Camera2D {
    pub fn new(viewport_width: f32, viewport_height: f32) -> Self {

        Self {
           view_proj: glam::Mat4::orthographic_lh(0.0,viewport_width, viewport_height, 0.0, 0.0, 1.0)
        }
    }


    pub fn to_matrix(&self) -> glam::Mat4 {
        self.view_proj
    } 
}


#[repr(C)]
#[derive(bytemuck::Zeroable, bytemuck::Pod, Copy, Clone)]
pub(crate) struct CameraUniform {
    view_proj: glam::Mat4
}

impl CameraUniform {
    pub fn from_matrix(matrix: glam::Mat4) -> Self {
        Self {
            view_proj: matrix
        }
    }
}