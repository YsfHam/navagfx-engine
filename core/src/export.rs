pub mod graphics_export {
    pub use wgpu::Color;
    pub use wgpu::SurfaceError;
}

pub mod application_export {
    pub use winit::keyboard::KeyCode;
}

pub use glam;
pub use image;