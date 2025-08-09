use wgpu::{include_wgsl, util::DeviceExt};

use crate::graphics::GraphicsContext;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable, bytemuck::Pod)]
struct Vertex {
    position: [f32; 2],
}

impl Vertex {

    const ATTRIBS: [wgpu::VertexAttribute; 1] =
        wgpu::vertex_attr_array![0 => Float32x2];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

const QUAD: &[Vertex] = &[
    Vertex { position: [-0.5, -0.5] },
    Vertex { position: [0.5, -0.5] },
    Vertex { position: [0.5, 0.5] },
    Vertex { position: [-0.5, 0.5] },
];

const QUAD_INDICES: &[u16] = &[
    0, 1, 2,
    2, 3, 0
];


pub struct Renderer2D {
    render_pipeline: wgpu::RenderPipeline,
    clear_color: wgpu::Color,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}


impl Renderer2D {
    pub fn new(context: &GraphicsContext) -> Self {
        let shader = context.device
                .create_shader_module(include_wgsl!("../../assets/shaders/shader_quad.wgsl"));

        let render_pipeline_layout = context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Renderer2D pipeline layout"),
            bind_group_layouts: &[],
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
                    Vertex::desc()
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

        Self {
            render_pipeline,
            clear_color: wgpu::Color {r: 0.1, g: 0.1, b: 0.2, a: 1.0},
            vertex_buffer: Self::create_vertex_buffer(context),
            index_buffer: Self::create_index_buffer(context)
        }
    }

    pub fn begin(&mut self, clear_color: wgpu::Color) {
        self.clear_color = clear_color;
    }

    pub fn submit(&self, context: &GraphicsContext) -> Result<(), wgpu::SurfaceError> {
        let output = context.surface.get_current_texture()?;
        let view = output.texture.create_view(&Default::default());

        let mut encoder = context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Renderer2D commands encoder"),
        });


        self.start_render_pass(&mut encoder, &view);

        context.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }


    fn start_render_pass(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {

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
        
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        render_pass.draw_indexed(0..QUAD_INDICES.len() as _, 0, 0..1);
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