pub mod renderer2d;
pub mod camera;
pub mod shapes;


use wgpu::SurfaceTarget;

pub struct GraphicsContext<'a> {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'a>,
    pub config: wgpu::SurfaceConfiguration,
}


impl<'a> GraphicsContext<'a> {
    pub async fn new(surface_target: impl Into<SurfaceTarget<'a>>, surface_width: u32, surface_height: u32) -> Self {


        log::info!("Creating instance");

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });


        log::info!("Creating surface");
        let surface = instance.create_surface(surface_target).unwrap();


        log::info!("Requesting Adapter");
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })
        .await
        .unwrap();


        log::info!("Requesting device and queue");

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("Graphics context device"),
            required_features: wgpu::Features::TEXTURE_BINDING_ARRAY,
            required_limits: wgpu::Limits::defaults(),
            memory_hints: Default::default(),
            trace: wgpu::Trace::Off,
        })
        .await
        .unwrap();


        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps.formats.iter()
        .find(|format| format.is_srgb())
        .copied()
        .unwrap_or(surface_caps.formats[0]);


        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: surface_width,
            height: surface_height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        log::info!("Configuring the surface");


        surface.configure(&device, &config);


        Self {
            config,
            device,
            queue,
            surface,
        }

    }


    pub(crate) fn resize_surface(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }
}