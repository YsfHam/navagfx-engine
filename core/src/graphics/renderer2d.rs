use std::{cell::{Cell, RefCell}, collections::HashMap};

use wgpu::{include_wgsl, util::DeviceExt};

use crate::{application::GraphicsContextRef, assets::{texture::{RawRgbaImageData, Texture2D, Texture2DCoordinates}, AssetHandle, AssetsManagerRef}, graphics::{camera::{Camera2D, CameraUniform}, shapes::Quad, GraphicsContext}};

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
    color: [f32; 4],
    tex_coords_size: [f32; 2],
    tex_coords_offset: [f32; 2],
}

impl QuadInstanceData {

    const ATTRIBS: [wgpu::VertexAttribute; 7] =
        wgpu::vertex_attr_array![2 => Float32x4, 3 => Float32x4, 4 => Float32x4, 5 => Float32x4, 6 => Float32x4, 7 => Float32x2, 8 => Float32x2];

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
    instance_buffer: RefCell<Option<wgpu::Buffer>>,
    buffer_len: Cell<usize>,
}

impl QuadsInstanceDataBuffer {
    fn new(quads_capacity: usize) -> Self {
        let quads = Vec::with_capacity(quads_capacity);
        Self {
            quads,
            instance_buffer: RefCell::new(None),
            buffer_len: Cell::new(0)
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

        if self.instance_buffer.borrow().is_none() {
            self.reallocate_instance_buffer(context);
        }
        else if self.buffer_len.get() < self.quads.len() {
            log::info!("Destroying instance buffer");
            self.instance_buffer.borrow().as_ref().unwrap().destroy();
            self.reallocate_instance_buffer(context);
        }
        else {
            context.queue.write_buffer(self.instance_buffer.borrow().as_ref().unwrap(), 0, bytemuck::cast_slice(&self.quads));
        }

        let instance_buffer = self.instance_buffer.borrow();


        render_pass.set_vertex_buffer(1, instance_buffer.as_ref().unwrap().slice(0..(self.quads.len() * std::mem::size_of::<QuadInstanceData>()) as _));
        render_pass.draw_indexed(0..QUAD_INDICES.len() as _, 0, 0..self.quads.len() as _);
    }

    fn reallocate_instance_buffer(&self, context: &GraphicsContext) {
        log::info!("Reallocating the instance buffer");
        let instance_buffer = context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&self.quads),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        self.instance_buffer.replace(Some(instance_buffer));
        self.buffer_len.set(self.quads.len());
    }
}

pub struct Renderer2D {
    render_pipeline: wgpu::RenderPipeline,
    assets_manager: AssetsManagerRef,
    context: GraphicsContextRef<'static>,
    clear_color: wgpu::Color,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,

    camera_uniform: Option<CameraUniform>,
    camera_buffer: wgpu::Buffer,
    camera_bind_group_layout: wgpu::BindGroupLayout,
    
    white_texture: AssetHandle<Texture2D>,
    quads_instances: HashMap<(AssetHandle<Texture2D>, i32), QuadsInstanceDataBuffer>,
}


impl Renderer2D {

    const MAX_QUAD: usize = 1_000_00;

    pub fn new(context: GraphicsContextRef<'static>, assets_manager: AssetsManagerRef) -> Self {
        let context_lock = context.read().unwrap();

        let shader = context_lock.device
                .create_shader_module(include_wgsl!("../../assets/shaders/shader_quad.wgsl"));


        let vertex_buffer = Self::create_vertex_buffer(&context_lock);
        let index_buffer = Self::create_index_buffer(&context_lock);
        
        let camera_bind_group_layout = context_lock.device
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

        let render_pipeline_layout = context_lock.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Renderer2D pipeline layout"),
            bind_group_layouts: &[
                &camera_bind_group_layout,
                &Texture2D::create_bind_group_layout(&context_lock)
            ],
            push_constant_ranges: &[],
        });

        let render_pipeline = context_lock.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                    format: context_lock.config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            })
        });


        let camera_buffer = context_lock.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Renderer2D camera buffer"),
            size: std::mem::size_of::<CameraUniform>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        //let mut assets_mgr_lock = 
        let white_texture = assets_manager
            .write()
            .unwrap()
            .load_asset(RawRgbaImageData {
                pixels: &[255, 255, 255, 255],
                width: 1,
                height: 1,
            })
            .unwrap();

        drop(context_lock);



        Self {
            render_pipeline,
            clear_color: wgpu::Color {r: 0.1, g: 0.1, b: 0.2, a: 1.0},
            vertex_buffer,
            index_buffer,
            context,

            camera_buffer,
            camera_uniform: None,
            camera_bind_group_layout,
            
            assets_manager,

            quads_instances: HashMap::new(),
            white_texture,
        }
    }

    pub fn begin(&mut self, clear_color: wgpu::Color, camera: &Camera2D) {

        self.clear_color = clear_color;

        self.camera_uniform = Some(CameraUniform::from_matrix(camera.to_matrix()));
        self.quads_instances.values_mut().for_each(QuadsInstanceDataBuffer::clear);
    }

    pub fn draw_quad(&mut self, quad: &Quad) {
        self.draw_quad_textured(quad, self.white_texture, Default::default());
    }

    pub fn draw_quad_textured(&mut self, quad: &Quad, texture_handle: AssetHandle<Texture2D>, atlas_coords: Texture2DCoordinates) {
        let quads = 
                self
                .quads_instances
                .entry((texture_handle, quad.z_index))
                .or_insert_with(|| QuadsInstanceDataBuffer::new(Self::MAX_QUAD))
                ;

        quads.push(QuadInstanceData {
            model: quad.get_transform(),
            color: quad.color.into(),
            tex_coords_offset: atlas_coords.offset,
            tex_coords_size: atlas_coords.size
        });
    }

    pub fn submit(&self) -> Result<(), wgpu::SurfaceError> {
        let context = self.context.read().unwrap();

        let output = context.surface.get_current_texture()?;
        let view = output.texture.create_view(&Default::default());

        let mut encoder = context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Renderer2D commands encoder"),
        });


        self.start_render_pass(&context, &mut encoder, &view);

        context.queue.submit(std::iter::once(encoder.finish()));
        drop(context);
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

        let mut entries = self.quads_instances.iter().collect::<Vec<_>>();
        entries.sort_by_key(|((_, z), _)| z);

        for ((handle, _), quads) in &entries {

            let lock = self.assets_manager.read().unwrap();
            let texture= lock.get_asset(*handle);

            render_pass.set_bind_group(1, &texture.bind_group, &[]);

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