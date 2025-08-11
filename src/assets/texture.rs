use image::RgbaImage;

use crate::{assets::Asset, graphics::GraphicsContext};

pub struct Texture2D {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Asset for Texture2D {}

impl Texture2D {

    pub fn from_image(context: &GraphicsContext, label: &str, image: &RgbaImage) -> Self {
        let dimensions = image.dimensions();

        Self::from_memory(context, label, image, dimensions.0, dimensions.1)
    }


    pub fn from_memory(context: &GraphicsContext, label: &str, texture_data: &[u8], texture_width: u32, texture_height: u32) 
    -> Self
    {
        let texture_size = wgpu::Extent3d {
            width: texture_width,
            height: texture_height,
            depth_or_array_layers: 1,
        };


        let texture = context.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });


        context.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            texture_data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * texture_width),
                rows_per_image: Some(texture_height),
            },
            texture_size
        );


        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(&(label.to_owned() + " texture view")),
            dimension: Some(wgpu::TextureViewDimension::D2),
            ..Default::default()
        });
        let sampler = context.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }

    }
}