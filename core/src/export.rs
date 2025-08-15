pub mod graphics_export {
    pub use wgpu::Color;
    pub use wgpu::SurfaceError;
}

pub mod application_export {
    pub use winit::keyboard::KeyCode;
    pub use winit::window::WindowAttributes;
}

pub use glam;
pub use image;
pub use log;