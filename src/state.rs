use std::time::Duration;

use crate::{curve::Bezier, Vertex};
use wgpu::ColorTargetState;

pub struct State<'window> {
    window: &'window winit::window::Window,
    surface_config: wgpu::SurfaceConfiguration,
    surface: wgpu::Surface<'window>,
    device: wgpu::Device,
    queue: wgpu::Queue,

    pipelines: [wgpu::RenderPipeline; 2],
    current_pipeline: usize,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,

    num_indices: u32,
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
                    required_features: wgpu::Features::POLYGON_MODE_LINE,
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let surface_config = Self::create_surface_config(&surface, &adapter, &size);
        surface.configure(&device, &surface_config);

        let shader_module = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex Buffer"),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            size: 10 * 2048,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Index Buffer"),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            size: 10 * 1024,
            mapped_at_creation: false,
        });

        // println!("{:#?} {:#?}", vertices[0], vertices[1]);

        let pipelines = [
            Self::create_fill_render_pipeline(&device, &shader_module, &surface_config),
            Self::create_line_render_pipeline(&device, &shader_module, &surface_config),
        ];

        Self {
            window,
            surface_config,
            surface,
            device,
            queue,
            pipelines,
            current_pipeline: 0,
            vertex_buffer,
            index_buffer,
            num_indices: 0,
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
                view,
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

        render_pass.set_pipeline(&self.pipelines[self.current_pipeline]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
    }

    pub fn window(&self) -> &winit::window::Window {
        self.window
    }

    pub fn input(&mut self, _event: &mut winit::event::WindowEvent) -> bool {
        use winit::event::{ElementState, KeyEvent, WindowEvent};
        use winit::keyboard::{KeyCode, PhysicalKey};
        match _event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::Space),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                self.current_pipeline ^= 1;
                false
            }
            _ => false,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width == 0 && new_size.height == 0 {
            return;
        }
        self.surface_config.height = new_size.height;
        self.surface_config.width = new_size.width;
        self.surface.configure(&self.device, &self.surface_config);
    }

    pub fn update(&mut self, since_start: Duration) {
        let width = 0.01;
        let count = 30;

        let speed = 1000.0;

        let start_y = ((since_start.as_millis() as f64) / speed).sin() * 0.5;
        let middle_y = ((since_start.as_millis() as f64) / speed * 2.0).sin();
        let end_y = ((since_start.as_millis() as f64) / speed * 1.5).sin() * 0.5;

        let poly_line = Bezier::new(
            cgmath::Vector2 {
                x: -0.5,
                y: start_y,
            },
            cgmath::Vector2 {
                x: 0.0,
                y: middle_y,
            },
            cgmath::Vector2 { x: 0.5, y: end_y },
        )
        .subdivide(count);

        let renderer = crate::curve::renderer::TangentRenderer::new();
        let data = renderer.render(&poly_line, width);

        self.queue
            .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&data.vertices));
        self.queue
            .write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&data.indices));
        self.num_indices = data.indices.len() as u32;
    }

    fn create_fill_render_pipeline(
        device: &wgpu::Device,
        shader_module: &wgpu::ShaderModule,
        surface_config: &wgpu::SurfaceConfiguration,
    ) -> wgpu::RenderPipeline {
        let vertex = Self::create_vertex_state(shader_module);
        let color_targets = Self::create_color_targets(surface_config);
        let fragment = Self::create_fragment_state(shader_module, &color_targets);
        let primitive = Self::create_fill_primitive_state();
        let multisample = Self::create_multisample_state();

        let render_pipeline_layout = Self::create_pipeline_layout(device);
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

    fn create_line_render_pipeline(
        device: &wgpu::Device,
        shader_module: &wgpu::ShaderModule,
        surface_config: &wgpu::SurfaceConfiguration,
    ) -> wgpu::RenderPipeline {
        let vertex = Self::create_vertex_state(shader_module);
        let color_targets = Self::create_color_targets(surface_config);
        let fragment = Self::create_fragment_state(shader_module, &color_targets);
        let primitive = Self::create_line_primitive_state();
        let multisample = Self::create_multisample_state();

        let render_pipeline_layout = Self::create_pipeline_layout(device);
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

    fn create_surface_config(
        surface: &wgpu::Surface,
        adapter: &wgpu::Adapter,
        size: &winit::dpi::PhysicalSize<u32>,
    ) -> wgpu::SurfaceConfiguration {
        let surface_capabilities = surface.get_capabilities(adapter);
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
            present_mode: wgpu::PresentMode::AutoNoVsync,
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

    fn create_fragment_state<'a>(
        shader_module: &'a wgpu::ShaderModule,
        targets: &'a [Option<ColorTargetState>],
    ) -> wgpu::FragmentState<'a> {
        wgpu::FragmentState {
            module: shader_module,
            entry_point: "fs_main",
            targets,
        }
    }

    fn create_color_targets(
        surface_config: &wgpu::SurfaceConfiguration,
    ) -> Vec<Option<ColorTargetState>> {
        vec![Some(wgpu::ColorTargetState {
            format: surface_config.format,
            blend: Some(wgpu::BlendState::REPLACE),
            write_mask: wgpu::ColorWrites::ALL,
        })]
    }

    fn create_fill_primitive_state() -> wgpu::PrimitiveState {
        wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        }
    }

    fn create_line_primitive_state() -> wgpu::PrimitiveState {
        wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Line,
            unclipped_depth: false,
            conservative: false,
        }
    }

    fn create_multisample_state() -> wgpu::MultisampleState {
        wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        }
    }
}
