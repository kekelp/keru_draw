pub mod shapes;
pub use shapes::*;

pub mod gpu_vec;
use gpu_vec::GpuVec;
use std::time::Duration;


pub use textslabs;

pub use textslabs::{
    Text, TextRenderer, TextBoxHandle, TextEditHandle, QuadRanges,
    TextStyle2, StyleHandle, ColorBrush, with_clipboard, BoundingBox,
    parley,
};
// Re-export font properties from parley
pub use textslabs::parley::{FontWeight, FontStyle, LineHeight, FontStack};
pub use keru_images::{ImageRenderer, LoadedImage};
use wgpu_profiler::{GpuProfiler, GpuProfilerSettings};

pub use euclid;
use euclid::UnknownUnit;

pub mod primitive {
    pub const BOX: u32 = 0;
    pub const CIRCLE: u32 = 1;
    pub const SEGMENT: u32 = 2;
    pub const TEXT: u32 = 3;
    pub const IMAGE: u32 = 4;
}

/// A screen-space rectangle in pixel coordinates.
/// Screen space has (0, 0) at the top-left corner, with Y increasing downward.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenRect {
    /// Minimum X coordinate in pixels (left edge)
    pub min_x: f32,
    /// Minimum Y coordinate in pixels (top edge)
    pub min_y: f32,
    /// Maximum X coordinate in pixels (right edge)
    pub max_x: f32,
    /// Maximum Y coordinate in pixels (bottom edge)
    pub max_y: f32,
}

impl ScreenRect {
    /// Create a new ScreenRect from pixel coordinates.
    pub fn new(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Self {
        Self { min_x, min_y, max_x, max_y }
    }

    /// Convert to the tuple format expected by textslabs.
    fn to_tuple(self) -> (f32, f32, f32, f32) {
        (self.min_x, self.min_y, self.max_x, self.max_y)
    }
}

/// A 2D affine transform using euclid's Transform2D.
pub type Transform = euclid::Transform2D<f32, UnknownUnit, UnknownUnit>;

/// Combines a euclid Transform2D with a textslabs Transform2D.
/// The textslabs transform is applied first, then the euclid transform.
fn combine_transforms(euclid_transform: &Transform, textslabs_transform: &textslabs::Transform2D) -> textslabs::Transform2D {
    // Convert textslabs transform to euclid
    // textslabs applies: scale, then rotation, then translation
    let cos_r = textslabs_transform.rotation.cos();
    let sin_r = textslabs_transform.rotation.sin();
    let s = textslabs_transform.scale;

    let textslabs_euclid = Transform::new(
        s * cos_r, s * sin_r,
        -s * sin_r, s * cos_r,
        textslabs_transform.translation.0,
        textslabs_transform.translation.1,
    );

    // Combine: textslabs_euclid, then euclid_transform
    let combined_euclid = textslabs_euclid.then(euclid_transform);

    // Convert back to textslabs::Transform2D
    // Extract translation from m31, m32
    let translation = (combined_euclid.m31, combined_euclid.m32);

    // Extract rotation and scale from the matrix
    // For a 2D transform matrix: [[m11, m21], [m12, m22]]
    // rotation = atan2(m12, m11)
    // scale = sqrt(m11^2 + m12^2) (assuming uniform scale)
    let rotation = combined_euclid.m12.atan2(combined_euclid.m11);
    let scale = (combined_euclid.m11.powi(2) + combined_euclid.m12.powi(2)).sqrt();

    textslabs::Transform2D {
        translation,
        rotation,
        scale,
    }
}

pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
    pub image_renderer: ImageRenderer,
    pub text: Text,
    text_renderer: TextRenderer,
    shapes: Shapes,
    instances: GpuVec<Instance>,
    transforms: GpuVec<Transform>,
    transform_stack: Vec<usize>,
    pub gpu_profiler: GpuProfiler,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Instance {
    p_type: u32,
    p_index: u32,
    transform_index: u32,
    _padding: u32,
}

impl Renderer {
    pub fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        #[cfg(debug_assertions)] {
            assert_imported_image_shader_matches();
            assert_imported_textslabs_shader_matches();
        }

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

        let transforms_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("keru_draw transforms bind group layout"),
            entries: &[
                GpuVec::<Transform>::bind_group_layout_entry(0),
            ],
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                // The binding indices has to match the order in which the parameter blocks appear in the shader!
                // If there are issues, compile the shaders with the -reflection-json flag and see the parameterBlock fields.
                // Binding order: transformsData(0), shapes(1), textslabs(2), imageatlas(3)
                bind_group_layouts: &[
                    &transforms_bind_group_layout,
                    &Shapes::bind_group_layout(&device),
                    &text_renderer.bind_group_layout(),
                    svg_renderer.bind_group_layout(),
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
                    array_stride: 16,
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
                        wgpu::VertexAttribute {
                            offset: 8,
                            shader_location: 2,
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

        let instances = GpuVec::with_usage(
            &device,
            256,
            "keru_draw instances",
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        );

        let mut transforms = GpuVec::new(&device, 64, "keru_draw transforms");
        // Push identity transform at index 0
        transforms.push(Transform::identity());

        let gpu_profiler = GpuProfiler::new(&device, GpuProfilerSettings {
            enable_timer_queries: false,
            enable_debug_groups: false,
            max_num_pending_frames: 3,
        }).unwrap();

        Self {
            device,
            queue,
            render_pipeline,
            shapes,
            text,
            text_renderer,
            image_renderer: svg_renderer,
            instances,
            transforms,
            transform_stack: vec![0], // Start with identity transform at index 0
            gpu_profiler,
        }
    }

    // Shape drawing methods
    pub fn draw_box(
        &mut self,
        top_left: [f32; 2],
        size: [f32; 2],
        corner_radius: f32,
        border_thickness: f32,
        color: [f32; 4],
        x_clip: [f32; 2],
        y_clip: [f32; 2],
    ) {
        let index = self.shapes.push_box(top_left, size, corner_radius, border_thickness, color, x_clip, y_clip);
        self.instances.push(Instance {
            p_type: primitive::BOX,
            p_index: index as u32,
            transform_index: *self.transform_stack.last().unwrap() as u32,
            _padding: 0,
        });
    }

    pub fn draw_box_gradient(
        &mut self,
        top_left: [f32; 2],
        size: [f32; 2],
        corner_radius: f32,
        border_thickness: f32,
        color_start: [f32; 4],
        color_end: [f32; 4],
        gradient_angle: f32,
        x_clip: [f32; 2],
        y_clip: [f32; 2],
    ) {
        let index = self.shapes.push_box_gradient(top_left, size, corner_radius, border_thickness, color_start, color_end, gradient_angle, x_clip, y_clip);
        self.instances.push(Instance {
            p_type: primitive::BOX,
            p_index: index as u32,
            transform_index: *self.transform_stack.last().unwrap() as u32,
            _padding: 0,
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
            transform_index: *self.transform_stack.last().unwrap() as u32,
            _padding: 0,
        });
    }

    pub fn draw_circle_gradient(
        &mut self,
        center: [f32; 2],
        radius: f32,
        color_start: [f32; 4],
        color_end: [f32; 4],
        gradient_type: u32, // 1=linear, 2=radial
        gradient_angle: f32,
        x_clip: [f32; 2],
        y_clip: [f32; 2],
    ) {
        let index = self.shapes.push_circle_gradient(center, radius, color_start, color_end, gradient_type, gradient_angle, x_clip, y_clip);
        self.instances.push(Instance {
            p_type: primitive::CIRCLE,
            p_index: index as u32,
            transform_index: *self.transform_stack.last().unwrap() as u32,
            _padding: 0,
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
            transform_index: *self.transform_stack.last().unwrap() as u32,
            _padding: 0,
        });
    }

    pub fn draw_ring_gradient(
        &mut self,
        center: [f32; 2],
        inner_radius: f32,
        outer_radius: f32,
        color_start: [f32; 4],
        color_end: [f32; 4],
        gradient_type: u32, // 1=linear, 2=radial
        gradient_angle: f32,
        x_clip: [f32; 2],
        y_clip: [f32; 2],
    ) {
        let index = self.shapes.push_ring_gradient(center, inner_radius, outer_radius, color_start, color_end, gradient_type, gradient_angle, x_clip, y_clip);
        self.instances.push(Instance {
            p_type: primitive::CIRCLE,
            p_index: index as u32,
            transform_index: *self.transform_stack.last().unwrap() as u32,
            _padding: 0,
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
            transform_index: *self.transform_stack.last().unwrap() as u32,
            _padding: 0,
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
            transform_index: *self.transform_stack.last().unwrap() as u32,
            _padding: 0,
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
        dash_length: Option<f32>,
    ) {
        let index = self.shapes.push_segment(start, end, thickness, color, x_clip, y_clip, dash_length);
        self.instances.push(Instance {
            p_type: primitive::SEGMENT,
            p_index: index as u32,
            transform_index: *self.transform_stack.last().unwrap() as u32,
            _padding: 0,
        });
    }

    pub fn draw_segment_gradient(
        &mut self,
        start: [f32; 2],
        end: [f32; 2],
        thickness: f32,
        color_start: [f32; 4],
        color_end: [f32; 4],
        x_clip: [f32; 2],
        y_clip: [f32; 2],
        dash_length: Option<f32>,
    ) {
        let index = self.shapes.push_segment_gradient(start, end, thickness, color_start, color_end, x_clip, y_clip, dash_length);
        self.instances.push(Instance {
            p_type: primitive::SEGMENT,
            p_index: index as u32,
            transform_index: *self.transform_stack.last().unwrap() as u32,
            _padding: 0,
        });
    }

    /// Draw a text box.
    pub fn draw_text_box(&mut self, text_box: &TextBoxHandle) {
        // Get current transform before borrowing text_box_ref
        let current_euclid_transform = self.get_current_transform();

        // Combine keru_draw's transform with the text box's retained_transform
        let text_box_ref = self.text.get_text_box_mut(text_box);
        let retained = text_box_ref.retained_transform;

        // Combine: first apply retained_transform, then keru_draw's transform
        let combined = combine_transforms(&current_euclid_transform, &retained);
        text_box_ref.transform = combined;

        self.text_renderer.prepare_text_box_layout(text_box_ref);

        let QuadRanges { glyph_range, .. } = self.text.get_text_box(text_box).quad_range();

        for q in (glyph_range.0)..(glyph_range.1) {
            self.instances.push(Instance {
                p_type: primitive::TEXT,
                p_index: q as u32,
                transform_index: *self.transform_stack.last().unwrap() as u32,
                _padding: 0,
            });
        }
    }

    /// Draw a text edit widget.
    pub fn draw_text_edit(&mut self, text_edit: &TextEditHandle) {
        // Get current transform before borrowing text_edit_ref
        let current_euclid_transform = self.get_current_transform();

        // For TextEdit, we assume retained_transform is managed by the TextEdit itself
        // We just combine with the current transform
        let text_edit_ref = self.text.get_text_edit_mut(text_edit);

        // TextEdit doesn't expose retained_transform publicly
        // For now, we'll just apply the keru_draw transform on top of whatever transform the text_edit has
        // This might need adjustment if TextEdit needs to track retained_transform separately
        let current_text_transform = text_edit_ref.transform();
        let combined = combine_transforms(&current_euclid_transform, &current_text_transform);
        text_edit_ref.set_transform(combined);

        self.text_renderer.prepare_text_edit_layout(text_edit_ref);

        let QuadRanges { glyph_range, .. } = self.text.get_text_edit(text_edit).quad_range();

        for q in (glyph_range.0)..(glyph_range.1) {
            self.instances.push(Instance {
                p_type: primitive::TEXT,
                p_index: q as u32,
                transform_index: *self.transform_stack.last().unwrap() as u32,
                _padding: 0,
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
                transform_index: *self.transform_stack.last().unwrap() as u32,
                _padding: 0,
            });
        }
    }

    pub fn clear(&mut self) {
        self.shapes.clear();
        self.image_renderer.clear();
        self.instances.clear();
        self.text_renderer.clear();
        self.transforms.clear();
        // Re-add identity transform at index 0
        self.transforms.push(Transform::identity());
        self.transform_stack.clear();
        self.transform_stack.push(0);
    }

    pub fn begin_frame(&mut self, width: f32, height: f32) {
        // Update text renderer resolution
        self.text_renderer.update_resolution(width, height);
        // Update SVG renderer resolution
        self.image_renderer.update_resolution(width, height);
        // Clear all buffers
        self.clear();
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

    /// Push a new transform onto the stack.
    /// The transform is applied in screen space after clipping.
    pub fn push_transform(&mut self, transform: Transform) {
        // Add the transform to the buffer
        let new_index = self.transforms.len();
        self.transforms.push(transform);

        // Push the new index onto the stack
        self.transform_stack.push(new_index);
    }

    /// Pop the current transform from the stack, returning to the previous transform.
    /// Panics if trying to pop the last transform (the identity transform).
    pub fn pop_transform(&mut self) {
        if self.transform_stack.len() <= 1 {
            panic!("Cannot pop the last transform from the stack");
        }
        self.transform_stack.pop();
    }

    /// Get the current transform being used for draw calls.
    fn get_current_transform(&self) -> Transform {
        let current_index = *self.transform_stack.last().unwrap();
        if current_index < self.transforms.len() {
            self.transforms[current_index]
        } else {
            Transform::identity()
        }
    }

    /// Render into a render pass.
    pub fn render(&mut self, render_pass: &mut wgpu::RenderPass) {
        let decorations_range = self.text.prepare_decorations(&mut self.text_renderer);
        for q in (decorations_range.0)..(decorations_range.1) {
            self.instances.push(Instance {
                p_type: primitive::TEXT,
                p_index: q as u32,
                transform_index: *self.transform_stack.last().unwrap() as u32,
                _padding: 0,
            });
        }

        // Upload resources to GPU
        self.shapes.load_to_gpu(&self.device, &self.queue);
        self.text_renderer.load_to_gpu(&self.device, &self.queue);
        self.image_renderer.load_to_gpu(&self.device, &self.queue);
        self.instances.load_to_gpu(&self.device, &self.queue);

        let transforms_changed = self.transforms.load_to_gpu(&self.device, &self.queue);
        let transforms_bind_group = if transforms_changed {
            let layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("keru_draw transforms bind group layout"),
                entries: &[
                    GpuVec::<Transform>::bind_group_layout_entry(0),
                ],
            });
            self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("keru_draw transforms bind group"),
                layout: &layout,
                entries: &[
                    self.transforms.bind_group_entry(0),
                ],
            })
        } else {
            self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("keru_draw transforms bind group"),
                layout: &self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("keru_draw transforms bind group layout"),
                    entries: &[
                        GpuVec::<Transform>::bind_group_layout_entry(0),
                    ],
                }),
                entries: &[
                    self.transforms.bind_group_entry(0),
                ],
            })
        };

        render_pass.set_pipeline(&self.render_pipeline);
        // The binding indices has to match the order in which the parameter blocks appear in the shader!
        // If there are issues, compile the shaders with the -reflection-json flag and see the parameterBlock fields.
        // Binding order: transformsData(0), shapes(1), textslabs(2), imageatlas(3)
        render_pass.set_bind_group(0, &transforms_bind_group, &[]);
        render_pass.set_bind_group(1, &self.shapes.bind_group, &[]);
        render_pass.set_bind_group(2, &self.text_renderer.bind_group(), &[]);
        render_pass.set_bind_group(3, self.image_renderer.bind_group(), &[]);
        render_pass.set_vertex_buffer(0, self.instances.buffer().slice(..));

        render_pass.draw(0..4, 0..self.instances.len() as u32);
    }

    /// Convenience function that creates a render pass, renders into it, and presents to the screen.
    /// 
    /// Panics if the current surface texture can't be obtained from `surface`.  
    pub fn autorender(&mut self, surface: &wgpu::Surface, background_color: wgpu::Color) {
        let output = surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("keru_draw autorender render encoder"),
        });

        let query = self.gpu_profiler.begin_query("Render", &mut encoder);

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("keru_draw autorender render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(background_color),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            self.render(&mut render_pass);
        }

        self.gpu_profiler.end_query(&mut encoder, query);
        self.gpu_profiler.resolve_queries(&mut encoder);

        self.queue.submit(std::iter::once(encoder.finish()));

        self.gpu_profiler.end_frame().unwrap();

        if let Some(profiling_data) = self.gpu_profiler.process_finished_frame(self.queue.get_timestamp_period()) {
            for p in profiling_data {
                if let Some(time) = p.time {
                    let dur = Duration::from_secs_f64(time.end - time.start);
                    println!("Gpu time ({}): {:?} s", p.label, dur);
                }
            }
        }

        output.present();
    }
}

fn assert_imported_textslabs_shader_matches() {
    let imported_shader = include_str!("shaders/textslabs.slang");
    let original_shader = textslabs::TextRenderer::composable_shader_source();
    assert!(imported_shader == original_shader);
}

fn assert_imported_image_shader_matches() {
    let imported_shader = include_str!("shaders/keru_images.slang");
    let original_shader = keru_images::ImageRenderer::composable_shader_source();
    assert!(imported_shader == original_shader);
}

#[cfg(test)]
mod tests {
    use crate::*;
    #[test]
    fn test_imported_shaders() {
        assert_imported_textslabs_shader_matches();
        assert_imported_image_shader_matches();
    }
}