use winit::{
    event::WindowEvent,
    event_loop::EventLoop,
    window::Window,
};

mod rectangle;
mod ellipse;
mod globals;

use rectangle::{Rectangles, QuadData};
use ellipse::{Ellipses, EllipseData};
use globals::Globals;

struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    globals: Globals,
    rectangle_resources: Rectangles,
    ellipse_resources: Ellipses,
    num_quads: u32,
    num_ellipses: u32,
}

impl State {
    async fn new(window: std::sync::Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let vs_spirv = include_bytes!("../slangc_output/shader.vert.spv");
        let fs_spirv = include_bytes!("../slangc_output/shader.frag.spv");

        let vs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Vertex Shader"),
            source: wgpu::util::make_spirv(vs_spirv),
        });

        let fs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Fragment Shader"),
            source: wgpu::util::make_spirv(fs_spirv),
        });

        // Create bind group layouts for each parameter block
        let globals_layout = Globals::create_bind_group_layout(&device);
        let rectangle_layout = Rectangles::create_bind_group_layout(&device);
        let ellipse_layout = Ellipses::create_bind_group_layout(&device);

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&globals_layout, &rectangle_layout, &ellipse_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: Some("main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 8,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Uint32,
                        },
                        wgpu::VertexAttribute {
                            offset: 4,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Uint32,
                        },
                    ],
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: Some("main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
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
        });

        // Instance buffer - primitive type and primitive index per instance
        #[repr(C)]
        #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
        struct Instance {
            primitive_type: u32,
            primitive_index: u32,
        }

        let instances = [
            Instance { primitive_type: 0, primitive_index: 0 }, // Quad 0
            Instance { primitive_type: 1, primitive_index: 0 }, // Ellipse 0
        ];

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: std::mem::size_of_val(&instances) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        queue.write_buffer(&vertex_buffer, 0, bytemuck::cast_slice(&instances));

        // Create globals
        let mut globals = Globals::new(&device);
        globals.set_resolution(size.width as f32, size.height as f32);
        globals.upload(&queue);

        // Create resource modules
        let mut rectangles = Rectangles::new(&device);
        let mut ellipses = Ellipses::new(&device);

        // Add initial primitives (pixel coordinates)
        rectangles.push(QuadData {
            top_left: [100.0, 100.0],
            size: [300.0, 200.0],
            color: [1.0, 0.0, 0.0],
            corner_radius: 20.0,
            border_width: 2.0,
            border_color: [0.0, 0.0, 1.0],
        });

        ellipses.push(EllipseData {
            top_left: [500.0, 300.0],
            size: [150.0, 200.0],
            color: [0.0, 1.0, 0.0],
            _padding: 0.0,
        });

        // Upload to GPU
        rectangles.upload(&device, &queue);
        ellipses.upload(&device, &queue);

        let num_quads = rectangles.len() as u32;
        let num_ellipses = ellipses.len() as u32;

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            globals,
            rectangle_resources: rectangles,
            ellipse_resources: ellipses,
            num_quads,
            num_ellipses,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            // Update globals with new resolution
            self.globals.set_resolution(new_size.width as f32, new_size.height as f32);
            self.globals.upload(&self.queue);
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
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
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            // The binding indices has to match the order in which the parameter blocks appear in the shader!
            // If there are issues, compile the shaders with the -reflection-json flag and see the parameterBlock fields.
            render_pass.set_bind_group(0, &self.globals.bind_group, &[]);
            render_pass.set_bind_group(1, &self.ellipse_resources.bind_group, &[]);
            render_pass.set_bind_group(2, &self.rectangle_resources.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

            // Draw instanced quads: 4 vertices per quad, n instances
            let total_instances = self.num_quads + self.num_ellipses;
            render_pass.draw(0..4, 0..total_instances);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();

    let mut app = MainApp {
        window: None,
        state: None,
    };

    event_loop.run_app(&mut app).unwrap();
}

struct MainApp {
    window: Option<std::sync::Arc<Window>>,
    state: Option<State>,
}

impl winit::application::ApplicationHandler for MainApp {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_none() {
            let window = std::sync::Arc::new(
                event_loop
                    .create_window(Window::default_attributes())
                    .unwrap(),
            );
            let state = pollster::block_on(State::new(window.clone()));
            self.window = Some(window);
            self.state = Some(state);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if let (Some(window), Some(state)) = (&self.window, &mut self.state) {
            match event {
                WindowEvent::CloseRequested => event_loop.exit(),
                WindowEvent::Resized(physical_size) => {
                    state.resize(physical_size);
                }
                WindowEvent::RedrawRequested => {
                    match state.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                        Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                        Err(e) => eprintln!("{:?}", e),
                    }
                    window.request_redraw();
                }
                _ => {}
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
