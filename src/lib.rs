mod rectangle; pub use rectangle::*;
mod ellipse; pub use ellipse::*;
mod globals; pub use globals::*;

pub mod primitive {
    pub const RECTANGLE: u32 = 0;
    pub const ELLIPSE: u32 = 1;
}

pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
    instance_buffer: wgpu::Buffer,
    globals: Globals,
    rectangles: Rectangles,
    ellipses: Ellipses,
    instances: Vec<Instance>,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Instance {
    p_type: u32,
    p_index: u32,
}

impl Renderer {
    pub fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) -> Self {
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
        let mut globals = Globals::new(&device);
        globals.set_resolution(width as f32, height as f32);
        globals.upload(&queue);
        let globals_layout = Globals::bind_group_layout(&device);

        let rectangles = Rectangles::new(&device);
        let rectangle_layout = Rectangles::bind_group_layout(&device);
        
        let ellipses = Ellipses::new(&device);
        let ellipse_layout = Ellipses::bind_group_layout(&device);

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
                    format: surface_format,
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

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: 1024 * std::mem::size_of::<Instance>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            device,
            queue,
            render_pipeline,
            instance_buffer,
            globals,
            rectangles,
            ellipses,
            instances: Vec::new(),
        }
    }

    pub fn draw_rectangle(&mut self, data: RectangleData) {
        let index = self.rectangles.push(data);
        self.instances.push(Instance {
            p_type: primitive::RECTANGLE,
            p_index: index as u32,
        });
    }

    pub fn draw_ellipse(&mut self, data: EllipseData) {
        let index = self.ellipses.push(data);
        self.instances.push(Instance {
            p_type: primitive::ELLIPSE,
            p_index: index as u32,
        });
    }

    pub fn clear(&mut self) {
        self.rectangles.clear();
        self.ellipses.clear();
        self.instances.clear();
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.globals.set_resolution(width as f32, height as f32);
        self.globals.upload(&self.queue);
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn render(&mut self, view: &wgpu::TextureView) {
        // Upload resources to GPU
        self.rectangles.upload(&self.device, &self.queue);
        self.ellipses.upload(&self.device, &self.queue);

        // Update instance buffer
        if !self.instances.is_empty() {
            self.queue.write_buffer(
                &self.instance_buffer,
                0,
                bytemuck::cast_slice(&self.instances),
            );
        }

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
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
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            // The binding indices has to match the order in which the parameter blocks appear in the shader!
            // If there are issues, compile the shaders with the -reflection-json flag and see the parameterBlock fields.
            render_pass.set_bind_group(0, &self.globals.bind_group, &[]);
            render_pass.set_bind_group(1, &self.ellipses.bind_group, &[]);
            render_pass.set_bind_group(2, &self.rectangles.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.instance_buffer.slice(..));

            // Draw instanced quads: 4 vertices per quad, n instances
            render_pass.draw(0..4, 0..self.instances.len() as u32);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
    }
}

