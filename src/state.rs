use wgpu::{util::DeviceExt, ColorTargetState};
use crate::Vertex;

pub struct State<'window> {
    window: &'window winit::window::Window,
    surface_config: wgpu::SurfaceConfiguration,
    surface: wgpu::Surface<'window>,
    device: wgpu::Device,
    queue: wgpu::Queue,

    render_pipeline: wgpu::RenderPipeline,

    vertex_buffer: wgpu::Buffer,
}

impl<'window> State<'window> {
    pub async fn new(window: &'window winit::window::Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Wgpu device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let surface_config = Self::create_surface_config(&surface, &adapter, &size);
        surface.configure(&device, &surface_config);

        let shader_module = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let vertices = [
            Vertex::new_white([0.0, 0.0]),
            Vertex::new_white([1.0, 0.0]),
            Vertex::new_white([1.0, 1.0]),
        ];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let render_pipeline = Self::create_render_pipeline(&device, &shader_module, &surface_config);

        Self {
            window,
            surface_config,
            surface,
            device,
            queue,
            render_pipeline,
            vertex_buffer
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Command Encoder"),
            });

        self.render_pass(&mut encoder, &view);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn render_pass(&mut self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..3, 0..1);
    } 

    pub fn window(&self) -> &winit::window::Window {
        &self.window
    }

    pub fn input(&mut self, _event: &mut winit::event::WindowEvent) -> bool {
        false
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width == 0 && new_size.height == 0 {
            return;
        }
        self.surface_config.height = new_size.height;
        self.surface_config.width = new_size.width;
        self.surface.configure(&self.device, &self.surface_config);
    }

    pub fn update(&mut self) {}

    fn create_render_pipeline(device: &wgpu::Device, shader_module: &wgpu::ShaderModule, surface_config: &wgpu::SurfaceConfiguration) -> wgpu::RenderPipeline {
        let vertex = Self::create_vertex_state(&shader_module);
        let color_targets = Self::create_color_targets(&surface_config);
        let fragment = Self::create_fragment_state(&shader_module, &color_targets);
        let primitive = Self::create_primitive_state();
        let multisample = Self::create_multisample_state();

        let render_pipeline_layout = Self::create_pipeline_layout(&device);
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex,
            fragment: Some(fragment),
            primitive,
            depth_stencil: None,
            multisample,
            multiview: None,
        })
    }

    fn create_pipeline_layout(device: &wgpu::Device) -> wgpu::PipelineLayout {
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        })
    }

    fn create_surface_config(surface: &wgpu::Surface, adapter: &wgpu::Adapter, size: &winit::dpi::PhysicalSize<u32>) -> wgpu::SurfaceConfiguration {
        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_capabilities.formats[0]);
        wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
        }
    }

    const VERTEX_BUFFERS: [wgpu::VertexBufferLayout<'static>; 1] = [Vertex::desc()];

    fn create_vertex_state(shader_module: &wgpu::ShaderModule) -> wgpu::VertexState {
        wgpu::VertexState {
            module: shader_module,
            entry_point: "vs_main",
            buffers: &Self::VERTEX_BUFFERS,
        }
    }

    fn create_fragment_state<'a>(shader_module: &'a wgpu::ShaderModule, targets: &'a [Option<ColorTargetState>]) -> wgpu::FragmentState<'a> {
        wgpu::FragmentState {
            module: shader_module,
            entry_point: "fs_main",
            targets,
        }
    }

    fn create_color_targets(surface_config: &wgpu::SurfaceConfiguration) -> Vec<Option<ColorTargetState>> {
        vec![Some(wgpu::ColorTargetState {
            format: surface_config.format,
            blend: Some(wgpu::BlendState::REPLACE),
            write_mask: wgpu::ColorWrites::ALL,
        })]
    }

    fn create_primitive_state() -> wgpu::PrimitiveState {
        wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        }
    }

    fn create_multisample_state() -> wgpu::MultisampleState {
        wgpu::MultisampleState { count: 1, mask: !0, alpha_to_coverage_enabled: false }
    }
}
