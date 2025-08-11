use std::collections::HashMap;

use wgpu::{include_wgsl, util::DeviceExt};

use crate::{assets::{texture::Texture2D, AssetHandle, AssetsManagerRef}, graphics::{camera::{Camera2D, CameraUniform}, shapes::Quad, GraphicsContext}};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable, bytemuck::Pod)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

impl Vertex {

    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}


#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable, bytemuck::Pod)]
struct QuadInstanceData {
    model: glam::Mat4,
    color: [f32; 4]
}

impl QuadInstanceData {

    const ATTRIBS: [wgpu::VertexAttribute; 5] =
        wgpu::vertex_attr_array![2 => Float32x4, 3 => Float32x4, 4 => Float32x4, 5 => Float32x4, 6 => Float32x4];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

const QUAD: &[Vertex] = &[
    Vertex { position: [0.0, 0.0], tex_coords: [0.0, 0.0] },
    Vertex { position: [0.0, 1.0], tex_coords: [0.0, 1.0] },
    Vertex { position: [1.0, 1.0], tex_coords: [1.0, 1.0] },
    Vertex { position: [1.0, 0.0], tex_coords: [1.0, 0.0] },
];

const QUAD_INDICES: &[u16] = &[
    0, 1, 2,
    2, 3, 0
];


struct QuadsInstanceDataBuffer {
    quads: Vec<QuadInstanceData>,
}

impl QuadsInstanceDataBuffer {
    fn new(quads_capacity: usize) -> Self {
        let quads = Vec::with_capacity(quads_capacity);
        Self {
            quads,
        }
    }

    fn clear(&mut self) {
        self.quads.clear();
    }

    fn push(&mut self, quad: QuadInstanceData) {
        self.quads.push(quad);
    }

    fn submit_to_render_pass(&self, context: &GraphicsContext, render_pass: &mut wgpu::RenderPass) {
        if self.quads.is_empty() {
            return;
        }

        let instance_buffer = context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&self.quads),
            usage: wgpu::BufferUsages::VERTEX,
        });


        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
        render_pass.draw_indexed(0..QUAD_INDICES.len() as _, 0, 0..self.quads.len() as _);
    }
}

pub struct Renderer2D {
    render_pipeline: wgpu::RenderPipeline,
    assets_manager: AssetsManagerRef,
    clear_color: wgpu::Color,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,

    camera_uniform: Option<CameraUniform>,
    camera_buffer: wgpu::Buffer,
    camera_bind_group_layout: wgpu::BindGroupLayout,
    

    texture_bind_group_layout: wgpu::BindGroupLayout,
    white_texture: AssetHandle<Texture2D>,
    quads_instances: HashMap<AssetHandle<Texture2D>, QuadsInstanceDataBuffer>,
}


impl Renderer2D {

    const MAX_QUAD: usize = 1_000_00;

    pub fn new(context: &GraphicsContext, assets_manager: AssetsManagerRef) -> Self {
        let shader = context.device
                .create_shader_module(include_wgsl!("../../assets/shaders/shader_quad.wgsl"));


        
        let camera_bind_group_layout = context.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Renderer2D bind group layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        }
                    ],
                });


        let texture_bind_group_layout = context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            });

        let render_pipeline_layout = context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Renderer2D pipeline layout"),
            bind_group_layouts: &[
                &camera_bind_group_layout,
                &texture_bind_group_layout
            ],
            push_constant_ranges: &[],
        });


        let render_pipeline = context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render2D pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[
                    Vertex::desc(),
                    QuadInstanceData::desc()
                ],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: context.config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            })
        });


        let camera_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Renderer2D camera buffer"),
            size: std::mem::size_of::<CameraUniform>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut assets_mgr_lock = assets_manager.lock().unwrap();
        let white_texture = assets_mgr_lock.store_asset(
            Texture2D::from_memory(context, "dymm", &[255, 255, 255, 255], 1, 1)
        );
        drop(assets_mgr_lock);

        Self {
            render_pipeline,
            clear_color: wgpu::Color {r: 0.1, g: 0.1, b: 0.2, a: 1.0},
            vertex_buffer: Self::create_vertex_buffer(context),
            index_buffer: Self::create_index_buffer(context),

            camera_buffer,
            camera_uniform: None,
            camera_bind_group_layout,
            
            assets_manager,

            quads_instances: HashMap::new(),
            texture_bind_group_layout,
            white_texture,
        }
    }

    pub fn begin(&mut self, clear_color: wgpu::Color, camera: &Camera2D) {

        self.clear_color = clear_color;

        self.camera_uniform = Some(CameraUniform::from_matrix(camera.to_matrix()));
        self.quads_instances.values_mut().for_each(QuadsInstanceDataBuffer::clear);
    }

    pub fn draw_quad(&mut self, quad: &Quad) {
        self.draw_quad_textured(quad, self.white_texture);
    }

    pub fn draw_quad_textured(&mut self, quad: &Quad, texture_handle: AssetHandle<Texture2D>) {
        let quads = 
                self
                .quads_instances
                .entry(texture_handle)
                .or_insert_with(|| QuadsInstanceDataBuffer::new(Self::MAX_QUAD))
                ;

        quads.push(QuadInstanceData {
            model: quad.get_transform(),
            color: quad.color.into(),
        });
    }

    pub fn submit(&self, context: &GraphicsContext) -> Result<(), wgpu::SurfaceError> {
        let output = context.surface.get_current_texture()?;
        let view = output.texture.create_view(&Default::default());

        let mut encoder = context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Renderer2D commands encoder"),
        });


        self.start_render_pass(context, &mut encoder, &view);

        context.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }


    fn start_render_pass(&self, context: &GraphicsContext, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {

        let mut render_pass= encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Renderer2D color render pass"),
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,    
                    },
                    depth_slice: None,
                })
            ],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });


        render_pass.set_pipeline(&self.render_pipeline);

        render_pass.set_bind_group(0, &self.create_camera_bind_group(context), &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        self.render_quads(context, &mut render_pass);

    }

    fn render_quads(&self, context: &GraphicsContext, render_pass: &mut wgpu::RenderPass) {
        let lock = self.assets_manager.lock().unwrap(); 

        for (handle, quads) in &self.quads_instances {

            let texture= lock.get_asset(*handle);
            let texture_bind_group = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Quads bind group"),
                layout: &self.texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture.view)
                    },

                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&texture.sampler)
                    }
                ],
            });

            render_pass.set_bind_group(1, &texture_bind_group, &[]);

            quads.submit_to_render_pass(context, render_pass);
        }
    }
    fn create_camera_bind_group(&self, context: &GraphicsContext) -> wgpu::BindGroup {

        context.queue.write_buffer(
            &self.camera_buffer, 0, 
            bytemuck::cast_slice(&[self.camera_uniform.unwrap()]));

        context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Renderer2D camera bind group"),
            layout: &self.camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.camera_buffer.as_entire_binding(),
                }
            ],
        })
    }

    fn create_vertex_buffer(context: &GraphicsContext) -> wgpu::Buffer {
        context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Renderer2D vertext buffer"),
            contents: bytemuck::cast_slice(QUAD),
            usage: wgpu::BufferUsages::VERTEX
        })
    }

    fn create_index_buffer(context: &GraphicsContext) -> wgpu::Buffer {
        context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Renderer2D index buffer"),
            contents: bytemuck::cast_slice(QUAD_INDICES),
            usage: wgpu::BufferUsages::INDEX
        })
    }

}