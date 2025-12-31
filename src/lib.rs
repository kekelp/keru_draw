pub mod shapes;
pub use shapes::*;

pub use textslabs::{Text, TextRenderer, TextBoxHandle, TextEditHandle, QuadRanges};
pub use keru_images::{ImageRenderer, LoadedImage};

pub mod primitive {
    pub const BOX: u32 = 0;
    pub const CIRCLE: u32 = 1;
    pub const SEGMENT: u32 = 2;
    pub const TEXT: u32 = 3;
    pub const IMAGE: u32 = 4;
}

pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
    instance_buffer: wgpu::Buffer,
    pub shapes: Shapes,
    pub text: Text,
    text_renderer: TextRenderer,
    pub image_renderer: ImageRenderer,
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
    ) -> Self {
        let vs_spirv = include_bytes!("../slangc_output/shader.vert.spv");
        let vs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Vertex Shader"),
            source: wgpu::util::make_spirv(vs_spirv),
        });
        
        let fs_spirv = include_bytes!("../slangc_output/shader.frag.spv");
        let fs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Fragment Shader"),
            source: wgpu::util::make_spirv(fs_spirv),
        });

        let shapes = Shapes::new(&device);

        let text_renderer = TextRenderer::new(&device, &queue, surface_format);
        let text = Text::new();

        let svg_renderer = ImageRenderer::new(&device, &queue, surface_format);

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                // The binding indices has to match the order in which the parameter blocks appear in the shader!
                // If there are issues, compile the shaders with the -reflection-json flag and see the parameterBlock fields.
                bind_group_layouts: &[
                    &Shapes::bind_group_layout(&device),
                    &text_renderer.bind_group_layout(),
                    svg_renderer.bind_group_layout()
                ],
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
            shapes,
            text,
            text_renderer,
            image_renderer: svg_renderer,
            instances: Vec::new(),
        }
    }

    // Shape drawing methods
    pub fn draw_box(
        &mut self,
        top_left: [f32; 2],
        size: [f32; 2],
        corner_radius: f32,
        color: [f32; 3],
        x_clip: [f32; 2],
        y_clip: [f32; 2],
    ) {
        let index = self.shapes.push_box(top_left, size, corner_radius, color, x_clip, y_clip);
        self.instances.push(Instance {
            p_type: primitive::BOX,
            p_index: index as u32,
        });
    }

    pub fn draw_circle(
        &mut self,
        center: [f32; 2],
        radius: f32,
        color: [f32; 4],
        x_clip: [f32; 2],
        y_clip: [f32; 2],
    ) {
        let index = self.shapes.push_circle(center, radius, color, x_clip, y_clip);
        self.instances.push(Instance {
            p_type: primitive::CIRCLE,
            p_index: index as u32,
        });
    }

    pub fn draw_ring(
        &mut self,
        center: [f32; 2],
        inner_radius: f32,
        outer_radius: f32,
        color: [f32; 4],
        x_clip: [f32; 2],
        y_clip: [f32; 2],
    ) {
        let index = self.shapes.push_ring(center, inner_radius, outer_radius, color, x_clip, y_clip);
        self.instances.push(Instance {
            p_type: primitive::CIRCLE,
            p_index: index as u32,
        });
    }

    pub fn draw_arc(
        &mut self,
        center: [f32; 2],
        radius: f32,
        start_angle: f32,
        end_angle: f32,
        thickness: f32,
        color: [f32; 4],
        x_clip: [f32; 2],
        y_clip: [f32; 2],
    ) {
        let index = self.shapes.push_arc(center, radius, start_angle, end_angle, thickness, color, x_clip, y_clip);
        self.instances.push(Instance {
            p_type: primitive::CIRCLE,
            p_index: index as u32,
        });
    }

    pub fn draw_pie(
        &mut self,
        center: [f32; 2],
        radius: f32,
        start_angle: f32,
        end_angle: f32,
        color: [f32; 4],
        x_clip: [f32; 2],
        y_clip: [f32; 2],
    ) {
        let index = self.shapes.push_pie(center, radius, start_angle, end_angle, color, x_clip, y_clip);
        self.instances.push(Instance {
            p_type: primitive::CIRCLE,
            p_index: index as u32,
        });
    }

    pub fn draw_segment(
        &mut self,
        start: [f32; 2],
        end: [f32; 2],
        thickness: f32,
        color: [f32; 4],
        x_clip: [f32; 2],
        y_clip: [f32; 2],
    ) {
        let index = self.shapes.push_segment(start, end, thickness, color, x_clip, y_clip);
        self.instances.push(Instance {
            p_type: primitive::SEGMENT,
            p_index: index as u32,
        });
    }

    pub fn draw_text_box(&mut self, text_box: &TextBoxHandle) {
        let QuadRanges { glyph_range, decorations_range } = self.text.get_text_box(text_box).quad_range();

        // Push glyph quads - directly reference quad indices from textslabs
        for q in (glyph_range.0)..(glyph_range.1) {
            self.instances.push(Instance {
                p_type: primitive::TEXT,
                p_index: q as u32,
            });
        }

        // Push decoration quads
        for q in (decorations_range.0)..(decorations_range.1) {
            self.instances.push(Instance {
                p_type: primitive::TEXT,
                p_index: q as u32,
            });
        }
    }

    pub fn draw_text_edit(&mut self, text_edit: &TextEditHandle) {
        let QuadRanges { glyph_range, decorations_range } = self.text.get_text_edit(text_edit).quad_range();

        // Push glyph quads - directly reference quad indices from textslabs
        for q in (glyph_range.0)..(glyph_range.1) {
            self.instances.push(Instance {
                p_type: primitive::TEXT,
                p_index: q as u32,
            });
        }

        // Push decoration quads
        for q in (decorations_range.0)..(decorations_range.1) {
            self.instances.push(Instance {
                p_type: primitive::TEXT,
                p_index: q as u32,
            });
        }
    }

    pub fn draw_image(
        &mut self,
        handle: &LoadedImage,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        depth: f32,
    ) {
        let start_idx = self.image_renderer.quads().len();
        self.image_renderer.draw_svg(handle, x, y, width, height, depth);
        let end_idx = self.image_renderer.quads().len();
        for q in start_idx..end_idx {
            self.instances.push(Instance {
                p_type: primitive::IMAGE,
                p_index: q as u32,
            });
        }
    }

    pub fn clear(&mut self) {
        self.shapes.clear();
        self.image_renderer.clear();
        self.instances.clear();
    }

    pub fn begin_frame(&mut self, width: f32, height: f32) {
        // Update text renderer resolution
        self.text_renderer.update_resolution(width, height);
        // Update SVG renderer resolution
        self.image_renderer.update_resolution(width, height);
        // Prepare text layouts (must be done before drawing text)
        // Note: This requires at least one window event to have been processed
        self.text.prepare_all(&mut self.text_renderer);
        // Clear all buffers
        self.clear();
    }

    pub fn text_mut(&mut self) -> &mut Text {
        &mut self.text
    }

    pub fn text_renderer_mut(&mut self) -> &mut TextRenderer {
        &mut self.text_renderer
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.text_renderer.update_resolution(width as f32, height as f32);
        self.image_renderer.update_resolution(width as f32, height as f32);
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn render(&mut self, view: &wgpu::TextureView) {
        // Upload resources to GPU
        self.shapes.upload(&self.device, &self.queue);
        self.text_renderer.load_to_gpu(&self.device, &self.queue);
        self.image_renderer.load_to_gpu(&self.device, &self.queue);

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
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            // The binding indices has to match the order in which the parameter blocks appear in the shader!
            // If there are issues, compile the shaders with the -reflection-json flag and see the parameterBlock fields.
            render_pass.set_bind_group(0, &self.shapes.bind_group, &[]);
            render_pass.set_bind_group(1, &self.text_renderer.bind_group(), &[]);
            render_pass.set_bind_group(2, self.image_renderer.bind_group(), &[]);
            render_pass.set_vertex_buffer(0, self.instance_buffer.slice(..));

            render_pass.draw(0..4, 0..self.instances.len() as u32);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn imported_textslabs_shader_matches() {
        let imported_shader = include_str!("shaders/text.slang");
        let original_shader = textslabs::TextRenderer::composable_shader_source();
        assert!(imported_shader == original_shader);
    }

    #[test]
    fn imported_image_shader_matches() {
        let imported_shader = include_str!("shaders/keru_images.slang");
        let original_shader = keru_images::ImageRenderer::composable_shader_source();
        assert!(imported_shader == original_shader);
    }
}