use image::RgbaImage;

use crate::{assets::{loaders, Asset, AssetHasDefaultLoader}, graphics::GraphicsContext, impl_default_loader};


pub struct RawRgbaImageData<'a> {
    pub pixels: &'a [u8],
    pub width: u32,
    pub height: u32
}


#[derive(Copy, Clone)]
pub struct Texture2DCoordinates {
    pub size: [f32; 2],
    pub offset: [f32; 2],
}

impl Texture2DCoordinates {
    pub fn new(top: f32, left: f32, bottom: f32, right: f32) -> Self {
        Self {
            size: [right, bottom],
            offset: [left, top]
        }
    }
}

impl Default for Texture2DCoordinates {
    fn default() -> Self {
        Self { size: [1.0, 1.0], offset: [0.0, 0.0] }
    }
}

pub struct SpriteSheetCoordinates {
    coords: Vec<Texture2DCoordinates>,
    cols: usize,
}

impl SpriteSheetCoordinates {
    pub fn new(texture: &Texture2D, sprite_size: (u32, u32)) -> Self {
        let (sprite_width, sprite_height) = sprite_size;

        let size = [
            sprite_width as f32 / texture.width as f32,
            sprite_height as f32 / texture.height as f32,
        ];

        let mut atlas_coords = vec![];

        let rows = texture.height / sprite_height;
        let cols = texture.width / sprite_width;

        for y in 0..rows {
            for x in 0..cols {
                let offset = [
                    (x * sprite_width) as f32 / texture.width as f32,
                    (y * sprite_height) as f32 / texture.height as f32
                ];

                atlas_coords.push(Texture2DCoordinates {
                    size, offset
                });

            }
        }

        Self {
            coords: atlas_coords,
            cols: cols as usize
        }
    }

    pub fn get_coords(&self, x: usize, y: usize) -> Option<Texture2DCoordinates> {
        self.get_coords_by_index(y * self.cols + x)
    }

    pub fn get_coords_by_index(&self, index: usize) -> Option<Texture2DCoordinates> {
        self.coords.get(index).copied()
    }
    
    pub fn len(&self) -> usize {
        self.coords.len()
    }
}

pub struct Texture2D {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub width: u32,
    pub height: u32,

    pub bind_group: wgpu::BindGroup
}

impl Asset for Texture2D {}

impl_default_loader!(
    Texture2D, loaders::Texture2DLoader, 
    ([] => &str),
    (['a] => RawRgbaImageData<'a>)
);



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


        let bind_group = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Quads bind group"),
            layout: &Self::create_bind_group_layout(context),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view)
                },

                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler)
                }
            ],
        });

        Self {
            texture,
            view,
            sampler,
            width: texture_width,
            height: texture_height,
            bind_group
        }
    }

    pub fn create_bind_group_layout(context: &GraphicsContext) -> wgpu::BindGroupLayout {
        context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    // This should match the filterable field of the
                    // corresponding Texture entry above. 
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        })
    }
}