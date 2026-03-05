pub mod shapes;
pub use shapes::*;
pub mod gpu_vec;
pub mod images;

use gpu_vec::GpuVec;
use std::hash::{Hash, Hasher};
use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub struct InstanceRange { pub start: usize, pub end: usize }

pub use textslabs;

pub use textslabs::{
    Text, TextRenderer, TextBoxHandle, TextEditHandle, QuadRanges,
    TextStyle2, StyleHandle, ColorBrush, with_clipboard, BoundingBox,
    parley,
};
// Re-export font properties from parley
pub use textslabs::parley::{FontWeight, FontStyle, LineHeight, FontStack};
pub use images::{ImageRenderer, LoadedImage};
use wgpu_profiler::{GpuProfiler, GpuProfilerSettings};

pub use euclid;

pub mod primitive {
    pub const BOX: u32 = 0;
    pub const CIRCLE: u32 = 1;
    pub const SEGMENT: u32 = 2;
    pub const TEXT: u32 = 3;
    pub const GRID: u32 = 4;
    pub const TRIANGLE: u32 = 5;
    pub const HEXAGON: u32 = 6;
}

bitflags::bitflags! {
    /// Bitflags specifying which corners of a rectangle should be rounded.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct RoundedCorners: u32 {
        const TOP_LEFT = 1 << 0;
        const TOP_RIGHT = 1 << 1;
        const BOTTOM_LEFT = 1 << 2;
        const BOTTOM_RIGHT = 1 << 3;
        const ALL = Self::TOP_LEFT.bits() | Self::TOP_RIGHT.bits() | Self::BOTTOM_LEFT.bits() | Self::BOTTOM_RIGHT.bits();
        const NONE = 0;
        const TOP = Self::TOP_LEFT.bits() | Self::TOP_RIGHT.bits();
        const BOTTOM = Self::BOTTOM_LEFT.bits() | Self::BOTTOM_RIGHT.bits();
        const LEFT = Self::TOP_LEFT.bits() | Self::BOTTOM_LEFT.bits();
        const RIGHT = Self::TOP_RIGHT.bits() | Self::BOTTOM_RIGHT.bits();
    }
}

/// Gradient type for shapes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GradientType {
    Linear = 1,
    Radial = 2,
}

/// Gradient definition for shapes
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Gradient {
    pub color_start: [f32; 4],
    pub color_end: [f32; 4],
    pub gradient_type: GradientType,
    pub angle: f32,
}

impl Gradient {
    pub const fn new(color_start: [f32; 4], color_end: [f32; 4]) -> Self {
        Self {
            color_start,
            color_end,
            gradient_type: GradientType::Linear,
            angle: 0.0,
        }
    }

    pub const fn with_type(mut self, gradient_type: GradientType) -> Self {
        self.gradient_type = gradient_type;
        self
    }

    pub const fn with_angle(mut self, angle: f32) -> Self {
        self.angle = angle;
        self
    }

    pub const fn linear(color_start: [f32; 4], color_end: [f32; 4], angle: f32) -> Self {
        Self {
            color_start,
            color_end,
            gradient_type: GradientType::Linear,
            angle,
        }
    }

    pub const fn radial(color_start: [f32; 4], color_end: [f32; 4]) -> Self {
        Self {
            color_start,
            color_end,
            gradient_type: GradientType::Radial,
            angle: 0.0,
        }
    }
}

impl std::hash::Hash for Gradient {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.color_start[0].to_bits().hash(state);
        self.color_start[1].to_bits().hash(state);
        self.color_start[2].to_bits().hash(state);
        self.color_start[3].to_bits().hash(state);
        self.color_end[0].to_bits().hash(state);
        self.color_end[1].to_bits().hash(state);
        self.color_end[2].to_bits().hash(state);
        self.color_end[3].to_bits().hash(state);
        self.gradient_type.hash(state);
        self.angle.to_bits().hash(state);
    }
}

/// Fill style for shapes - solid color or gradient
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorFill {
    Color([f32; 4]),
    Gradient(Gradient),
}

impl Hash for ColorFill {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            ColorFill::Color(color) => {
                0u8.hash(state);
                color[0].to_bits().hash(state);
                color[1].to_bits().hash(state);
                color[2].to_bits().hash(state);
                color[3].to_bits().hash(state);
            },
            ColorFill::Gradient(gradient) => {
                1u8.hash(state);
                gradient.hash(state);
            },
        }
    }
}

/// Parameters for drawing a box/rectangle
#[derive(Debug, Clone)]
pub struct Box {
    pub top_left: [f32; 2],
    pub size: [f32; 2],
    pub corner_radius: f32,
    pub rounded_corners: RoundedCorners,
    pub border_thickness: f32,
    pub fill: ColorFill,
    pub x_clip: [f32; 2],
    pub y_clip: [f32; 2],
    pub texture: Option<LoadedImage>,
}

/// Parameters for drawing a circle
#[derive(Debug, Clone)]
pub struct Circle {
    pub center: [f32; 2],
    pub radius: f32,
    pub fill: ColorFill,
    pub x_clip: [f32; 2],
    pub y_clip: [f32; 2],
    pub texture: Option<LoadedImage>,
}

/// Parameters for drawing a ring (hollow circle)
#[derive(Debug, Clone)]
pub struct CircleRing {
    pub center: [f32; 2],
    pub inner_radius: f32,
    pub outer_radius: f32,
    pub fill: ColorFill,
    pub x_clip: [f32; 2],
    pub y_clip: [f32; 2],
    pub texture: Option<LoadedImage>,
    pub dash_length: Option<f32>,
}

/// Parameters for drawing an arc
#[derive(Debug, Clone)]
pub struct CircleArc {
    pub center: [f32; 2],
    pub radius: f32,
    pub start_angle: f32,
    pub end_angle: f32,
    pub thickness: f32,
    pub fill: ColorFill,
    pub x_clip: [f32; 2],
    pub y_clip: [f32; 2],
    pub texture: Option<LoadedImage>,
    pub dash_length: Option<f32>,
}

/// Parameters for drawing a pie slice
#[derive(Debug, Clone)]
pub struct CirclePie {
    pub center: [f32; 2],
    pub radius: f32,
    pub start_angle: f32,
    pub end_angle: f32,
    pub fill: ColorFill,
    pub x_clip: [f32; 2],
    pub y_clip: [f32; 2],
    pub texture: Option<LoadedImage>,
}

/// Parameters for drawing a line segment
#[derive(Debug, Clone)]
pub struct Segment {
    pub start: [f32; 2],
    pub end: [f32; 2],
    pub thickness: f32,
    pub fill: ColorFill,
    pub x_clip: [f32; 2],
    pub y_clip: [f32; 2],
    pub dash_length: Option<f32>,
    pub texture: Option<LoadedImage>,
}

/// Grid type for the grid primitive
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridType {
    Square = 0,
    Hexagonal = 1,
}

/// Parameters for drawing a grid
#[derive(Debug, Clone)]
pub struct Grid {
    pub top_left: [f32; 2],
    pub size: [f32; 2],
    pub lattice_size: f32,
    pub offset: [f32; 2],
    pub line_thickness: f32,
    pub color: [f32; 4],
    pub grid_type: GridType,
    pub x_clip: [f32; 2],
    pub y_clip: [f32; 2],
    pub texture: Option<LoadedImage>,
}

/// Parameters for drawing a triangle
#[derive(Debug, Clone)]
pub struct Triangle {
    pub p0: [f32; 2],
    pub p1: [f32; 2],
    pub p2: [f32; 2],
    pub fill: ColorFill,
    pub x_clip: [f32; 2],
    pub y_clip: [f32; 2],
    pub texture: Option<LoadedImage>,
}

/// Parameters for drawing a hexagon
#[derive(Debug, Clone)]
pub struct Hexagon {
    pub center: [f32; 2],
    pub size: f32,              // distance from center to vertex
    pub rotation: f32,          // rotation in radians (0 = flat-top)
    pub fill: ColorFill,
    pub stroke_thickness: f32,  // 0 = filled, >0 = stroke only
    pub x_clip: [f32; 2],
    pub y_clip: [f32; 2],
    pub texture: Option<LoadedImage>,
}

fn fill_gpu(fill: ColorFill) -> ([f32; 2], [f32; 4], [f32; 4], u32) {
    match fill {
        ColorFill::Color(color) => ([1.0, 0.0], color, color, 0),
        ColorFill::Gradient(g) => {
            ([g.angle.cos(), g.angle.sin()], g.color_start, g.color_end, g.gradient_type as u32)
        }
    }
}

fn texture_gpu(texture: Option<LoadedImage>) -> ([f32; 2], [f32; 2], u32) {
    let (texture_uv_origin, texture_uv_size, texture_page) = match texture {
        Some(image) => (
            [image.alloc.rectangle.min.x as f32, image.alloc.rectangle.min.y as f32],
            [image.width as f32, image.height as f32],
            image.page as u32,
        ),
        None => ([0.0, 0.0], [0.0, 0.0], u32::MAX),
    };
    return (texture_uv_origin, texture_uv_size, texture_page);
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

/// A simple 2D transform with uniform scale and offset
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Transform {
    pub offset: [f32; 2],
    pub scale: f32,
    pub _padding: f32,  // For 16-byte alignment
}

impl Transform {
    /// Create an identity transform (no translation, scale = 1.0)
    pub fn identity() -> Self {
        Self {
            offset: [0.0, 0.0],
            scale: 1.0,
            _padding: 0.0,
        }
    }

    /// Create a translation transform
    pub fn translation(x: f32, y: f32) -> Self {
        Self {
            offset: [x, y],
            scale: 1.0,
            _padding: 0.0,
        }
    }

    /// Create a scale transform centered at origin
    pub fn scale(scale: f32) -> Self {
        Self {
            offset: [0.0, 0.0],
            scale,
            _padding: 0.0,
        }
    }
}

/// Combines a keru_draw Transform with a textslabs Transform2D.
fn combine_transforms(keru_transform: &Transform, textslabs_transform: &textslabs::Transform2D) -> textslabs::Transform2D {
    textslabs::Transform2D {
        translation: (
            textslabs_transform.translation.0 + keru_transform.offset[0],
            textslabs_transform.translation.1 + keru_transform.offset[1],
        ),
        rotation: textslabs_transform.rotation,
        scale: textslabs_transform.scale * keru_transform.scale,
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
    transforms: GpuVec<Transform>,
    shapes_bind_group: wgpu::BindGroup,
    instances: GpuVec<Instance>,
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

        let text_renderer = TextRenderer::new(&device, &queue, surface_format);
        let text = Text::new();

        let shapes = Shapes::new(&device);
        let image_renderer = ImageRenderer::new(&device, &queue, surface_format);

        // Create transforms buffer with identity transform at index 0
        let mut transforms = GpuVec::new(&device, 64, "keru_draw transforms");
        transforms.push(Transform::identity());

        // Create merged bind group layout for shapes + images
        let shapes_bind_group_layout = Self::create_shapes_bind_group_layout(&device);

        // Create merged bind group
        let shapes_bind_group = Self::create_shapes_bind_group(
            &device,
            &shapes_bind_group_layout,
            &transforms,
            &shapes,
            &image_renderer,
        );

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                // The binding indices has to match the order in which the parameter blocks appear in the shader!
                // If there are issues, compile the shaders with the -reflection-json flag and see the parameterBlock fields.
                // Binding order: shapes+images(0), textslabs(1)
                bind_group_layouts: &[
                    &shapes_bind_group_layout,
                    &text_renderer.bind_group_layout(),
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

        let features = device.features();
        let timestamp_queries_supported = features.contains(wgpu::Features::TIMESTAMP_QUERY)
            && features.contains(wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS);

        #[cfg(debug_assertions)]
        let enable_timer_queries = timestamp_queries_supported;
        #[cfg(not(debug_assertions))]
        let enable_timer_queries = false;

        let gpu_profiler = GpuProfiler::new(&device, GpuProfilerSettings {
            enable_timer_queries,
            enable_debug_groups: false,
            max_num_pending_frames: 3,
        }).unwrap();

        Self {
            device,
            queue,
            render_pipeline,
            shapes,
            transforms,
            image_renderer,
            text,
            text_renderer,
            shapes_bind_group,
            instances,
            transform_stack: vec![0], // Start with identity transform at index 0
            gpu_profiler,
        }
    }

    fn create_shapes_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        // todo: rewrite this
        let mut entries = vec![
            // Transforms buffer
            GpuVec::<Transform>::bind_group_layout_entry(0),
        ];

        // Add shapes resources (bindings 1-5)
        let mut shapes_entries = Shapes::bind_group_layout_entries();
        for entry in &mut shapes_entries {
            entry.binding += 1; // Shift by 1 since transforms is at 0
        }
        entries.extend(shapes_entries);

        // Add image atlas resources (bindings 7-8)
        entries.extend_from_slice(&[
            // Image atlas texture array
            wgpu::BindGroupLayoutEntry {
                binding: 7,
                visibility: wgpu::ShaderStages::VERTEX.union(wgpu::ShaderStages::FRAGMENT),
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2Array,
                    multisampled: false,
                },
                count: None,
            },
            // Sampler
            wgpu::BindGroupLayoutEntry {
                binding: 8,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ]);

        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("keru_draw shapes+images bind group layout"),
            entries: &entries,
        })
    }

    fn create_shapes_bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        transforms: &GpuVec<Transform>,
        shapes: &Shapes,
        image_renderer: &ImageRenderer,
    ) -> wgpu::BindGroup {
        let texture_view = image_renderer.texture_array.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Shapes+Images Bind Group"),
            layout,
            entries: &[
                transforms.bind_group_entry(0),
                shapes.boxes.bind_group_entry(1),
                shapes.circles.bind_group_entry(2),
                shapes.segments.bind_group_entry(3),
                shapes.grids.bind_group_entry(4),
                shapes.triangles.bind_group_entry(5),
                shapes.hexagons.bind_group_entry(6),
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: wgpu::BindingResource::Sampler(&image_renderer.sampler),
                },
            ],
        })
    }

    // Shape drawing methods
    pub fn draw_box(&mut self, params: Box) {
        let (gradient_direction, color_start, color_end, gradient_type) = fill_gpu(params.fill);

        let (texture_uv_origin, texture_uv_size, texture_page) = texture_gpu(params.texture);

        let index = self.shapes.boxes.len();
        self.shapes.boxes.push(shapes::BoxGpu {
            top_left: params.top_left,
            size: params.size,
            x_clip: params.x_clip,
            y_clip: params.y_clip,
            corner_radius: params.corner_radius,
            border_thickness: params.border_thickness,
            gradient_direction,
            color_start,
            color_end,
            gradient_type,
            rounded_corners: params.rounded_corners.bits(),
            texture_uv_origin,
            texture_uv_size,
            texture_page,
            pad: [0.0; 5],
        });
        self.instances.push(Instance {
            p_type: primitive::BOX,
            p_index: index as u32,
            transform_index: *self.transform_stack.last().unwrap() as u32,
            _padding: 0,
        });
    }

    /// Draw an image.
    /// This is a convenience method equivalent to draw_box with white fill and the image as texture.
    pub fn draw_image(
        &mut self,
        image: LoadedImage,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        x_clip: [f32; 2],
        y_clip: [f32; 2],
    ) {
        self.draw_box(Box {
            top_left: [x, y],
            size: [width, height],
            corner_radius: 0.0,
            rounded_corners: RoundedCorners::NONE,
            border_thickness: 0.0,
            fill: ColorFill::Color([1.0, 1.0, 1.0, 1.0]),
            x_clip,
            y_clip,
            texture: Some(image),
        });
    }

    pub fn draw_circle(&mut self, params: Circle) {
        let (gradient_direction, color_start, color_end, gradient_type) = fill_gpu(params.fill);

        let (texture_uv_origin, texture_uv_size, texture_page) = texture_gpu(params.texture);

        let index = self.shapes.circles.len();
        self.shapes.circles.push(shapes::CircleGpu {
            center: params.center,
            radii: [0.0, params.radius],
            angles: [0.0, std::f32::consts::TAU],
            x_clip: params.x_clip,
            y_clip: params.y_clip,
            gradient_direction,
            color_start,
            color_end,
            gradient_type,
            texture_page,
            texture_uv_origin,
            texture_uv_size,
            dash_length: 0.0,
            pad: 0.0,
        });
        self.instances.push(Instance {
            p_type: primitive::CIRCLE,
            p_index: index as u32,
            transform_index: *self.transform_stack.last().unwrap() as u32,
            _padding: 0,
        });
    }

    pub fn draw_ring(&mut self, params: CircleRing) {
        let (gradient_direction, color_start, color_end, gradient_type) = fill_gpu(params.fill);

        let (texture_uv_origin, texture_uv_size, texture_page) = texture_gpu(params.texture);

        let index = self.shapes.circles.len();
        self.shapes.circles.push(shapes::CircleGpu {
            center: params.center,
            radii: [params.inner_radius, params.outer_radius],
            angles: [0.0, std::f32::consts::TAU],
            x_clip: params.x_clip,
            y_clip: params.y_clip,
            gradient_direction,
            color_start,
            color_end,
            gradient_type,
            texture_page,
            texture_uv_origin,
            texture_uv_size,
            dash_length: params.dash_length.unwrap_or(0.0),
            pad: 0.0,
        });
        self.instances.push(Instance {
            p_type: primitive::CIRCLE,
            p_index: index as u32,
            transform_index: *self.transform_stack.last().unwrap() as u32,
            _padding: 0,
        });
    }

    pub fn draw_arc(&mut self, params: CircleArc) {
        let (gradient_direction, color_start, color_end, gradient_type) = fill_gpu(params.fill);

        let (texture_uv_origin, texture_uv_size, texture_page) = texture_gpu(params.texture);

        let index = self.shapes.circles.len();
        self.shapes.circles.push(shapes::CircleGpu {
            center: params.center,
            radii: [params.radius - params.thickness * 0.5, params.radius + params.thickness * 0.5],
            angles: [params.start_angle, params.end_angle],
            x_clip: params.x_clip,
            y_clip: params.y_clip,
            gradient_direction,
            color_start,
            color_end,
            gradient_type,
            texture_page,
            texture_uv_origin,
            texture_uv_size,
            dash_length: params.dash_length.unwrap_or(0.0),
            pad: 0.0,
        });
        self.instances.push(Instance {
            p_type: primitive::CIRCLE,
            p_index: index as u32,
            transform_index: *self.transform_stack.last().unwrap() as u32,
            _padding: 0,
        });
    }

    pub fn draw_pie(&mut self, params: CirclePie) {
        let (gradient_direction, color_start, color_end, gradient_type) = fill_gpu(params.fill);

        let (texture_uv_origin, texture_uv_size, texture_page) = texture_gpu(params.texture);

        let index = self.shapes.circles.len();
        self.shapes.circles.push(shapes::CircleGpu {
            center: params.center,
            radii: [0.0, params.radius],
            angles: [params.start_angle, params.end_angle],
            x_clip: params.x_clip,
            y_clip: params.y_clip,
            gradient_direction,
            color_start,
            color_end,
            gradient_type,
            texture_page,
            texture_uv_origin,
            texture_uv_size,
            dash_length: 0.0,
            pad: 0.0,
        });
        self.instances.push(Instance {
            p_type: primitive::CIRCLE,
            p_index: index as u32,
            transform_index: *self.transform_stack.last().unwrap() as u32,
            _padding: 0,
        });
    }

    pub fn draw_segment(&mut self, params: Segment) {
        let (color_start, color_end, gradient_type) = match params.fill {
            ColorFill::Color(color) => (color, color, 0),
            ColorFill::Gradient(g) => {
                (g.color_start, g.color_end, 1)
            }
        };

        let (texture_uv_origin, texture_uv_size, texture_page) = texture_gpu(params.texture);

        let index = self.shapes.segments.len();
        self.shapes.segments.push(shapes::SegmentGpu {
            start: params.start,
            end: params.end,
            x_clip: params.x_clip,
            y_clip: params.y_clip,
            color_start,
            color_end,
            thickness_dash: [params.thickness, params.dash_length.unwrap_or(0.0), 1.0, 1.0],
            gradient_type,
            texture_page,
            texture_uv_origin,
            texture_uv_size,
            pad: [0.0; 2],
        });
        self.instances.push(Instance {
            p_type: primitive::SEGMENT,
            p_index: index as u32,
            transform_index: *self.transform_stack.last().unwrap() as u32,
            _padding: 0,
        });
    }

    pub fn draw_grid(&mut self, params: Grid) {
        let (texture_uv_origin, texture_uv_size, texture_page) = texture_gpu(params.texture);

        let index = self.shapes.grids.len();
        self.shapes.grids.push(shapes::GridGpu {
            top_left: params.top_left,
            size: params.size,
            x_clip: params.x_clip,
            y_clip: params.y_clip,
            offset: params.offset,
            lattice_size: params.lattice_size,
            line_thickness: params.line_thickness,
            color: params.color,
            grid_type: params.grid_type as u32,
            texture_page,
            texture_uv_origin,
            texture_uv_size,
            pad: [0.0, 0.0],
        });
        self.instances.push(Instance {
            p_type: primitive::GRID,
            p_index: index as u32,
            transform_index: *self.transform_stack.last().unwrap() as u32,
            _padding: 0,
        });
    }

    pub fn draw_triangle(&mut self, params: Triangle) {
        let (gradient_direction, color_start, color_end, gradient_type) = fill_gpu(params.fill);

        let (texture_uv_origin, texture_uv_size, texture_page) = texture_gpu(params.texture);

        let index = self.shapes.triangles.len();
        self.shapes.triangles.push(shapes::TriangleGpu {
            p0: params.p0,
            p1: params.p1,
            p2: params.p2,
            x_clip: params.x_clip,
            y_clip: params.y_clip,
            gradient_direction,
            color_start,
            color_end,
            gradient_type,
            texture_page,
            texture_uv_origin,
            texture_uv_size,
            pad: [0.0; 2],
        });
        self.instances.push(Instance {
            p_type: primitive::TRIANGLE,
            p_index: index as u32,
            transform_index: *self.transform_stack.last().unwrap() as u32,
            _padding: 0,
        });
    }

    pub fn draw_hexagon(&mut self, params: Hexagon) {
        let (gradient_direction, color_start, color_end, gradient_type) = fill_gpu(params.fill);

        let (texture_uv_origin, texture_uv_size, texture_page) = texture_gpu(params.texture);

        let index = self.shapes.hexagons.len();
        self.shapes.hexagons.push(shapes::HexagonGpu {
            center: params.center,
            size: params.size,
            rotation: params.rotation,
            x_clip: params.x_clip,
            y_clip: params.y_clip,
            gradient_direction,
            color_start,
            color_end,
            gradient_type,
            stroke_thickness: params.stroke_thickness,
            texture_page,
            _pad1: 0.0,
            texture_uv_origin,
            texture_uv_size,
            _pad2: [0.0; 2],
        });
        self.instances.push(Instance {
            p_type: primitive::HEXAGON,
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

    pub fn clear(&mut self) {
        self.shapes.clear();
        self.instances.clear();
        self.text_renderer.clear();
        self.transforms.clear();
        self.transforms.push(Transform::identity());
        self.transform_stack.clear();
        self.transform_stack.push(0);
    }

    pub fn begin_frame(&mut self, width: f32, height: f32) {
        // Update text renderer resolution
        self.text_renderer.update_resolution(width, height);
        // Clear all buffers
        self.clear();
    }


    pub fn text_renderer_mut(&mut self) -> &mut TextRenderer {
        &mut self.text_renderer
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.text_renderer.update_resolution(width as f32, height as f32);
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// Returns the current number of instances that have been added so far.
    pub fn instance_count(&self) -> usize {
        self.instances.len()
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
        // Upload resources to GPU
        let transforms_changed = self.transforms.load_to_gpu(&self.device, &self.queue);
        let shapes_changed = self.shapes.load_to_gpu(&self.device, &self.queue);
        let images_changed = self.image_renderer.load_to_gpu(&self.device, &self.queue);

        // Recreate bind group if transforms, shapes or images changed
        if transforms_changed || shapes_changed || images_changed {
            let layout = Self::create_shapes_bind_group_layout(&self.device);
            self.shapes_bind_group = Self::create_shapes_bind_group(
                &self.device,
                &layout,
                &self.transforms,
                &self.shapes,
                &self.image_renderer,
            );
        }

        self.text_renderer.load_to_gpu(&self.device, &self.queue);
        self.instances.load_to_gpu(&self.device, &self.queue);

        self.set_pipeline_state(render_pass);

        render_pass.draw(0..4, 0..self.instances.len() as u32);
    }

    pub fn prepare_text_decorations(&mut self) {
        let decorations_range = self.text.prepare_decorations(&mut self.text_renderer);
        for q in (decorations_range.0)..(decorations_range.1) {
            self.instances.push(Instance {
                p_type: primitive::TEXT,
                p_index: q as u32,
                transform_index: *self.transform_stack.last().unwrap() as u32,
                _padding: 0,
            });
        }
    }

    pub fn load_to_gpu(&mut self) {
        // Upload resources to GPU
        let transforms_changed = self.transforms.load_to_gpu(&self.device, &self.queue);
        let shapes_changed = self.shapes.load_to_gpu(&self.device, &self.queue);
        let images_changed = self.image_renderer.load_to_gpu(&self.device, &self.queue);

        // Recreate bind group if transforms, shapes or images changed
        if transforms_changed || shapes_changed || images_changed {
            let layout = Self::create_shapes_bind_group_layout(&self.device);
            self.shapes_bind_group = Self::create_shapes_bind_group(
                &self.device,
                &layout,
                &self.transforms,
                &self.shapes,
                &self.image_renderer,
            );
        }

        self.text_renderer.load_to_gpu(&self.device, &self.queue);
        self.instances.load_to_gpu(&self.device, &self.queue);
    }

    pub fn set_pipeline_state(&mut self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_pipeline(&self.render_pipeline);
        // The binding indices has to match the order in which the parameter blocks appear in the shader!
        // If there are issues, compile the shaders with the -reflection-json flag and see the parameterBlock fields.
        // Binding order: shapes+images(0), textslabs(1)
        render_pass.set_bind_group(0, &self.shapes_bind_group, &[]);
        render_pass.set_bind_group(1, &self.text_renderer.bind_group(), &[]);
        render_pass.set_vertex_buffer(0, self.instances.buffer().slice(..));
    }

    /// Render a specific range of instances into a render pass.
    ///
    /// This is useful for custom rendering where you want to interleave
    /// Keru's rendering with your own custom drawing code.
    ///
    /// Note: You must call `setup_render_pass()` before calling this method.
    pub fn render_range(&mut self, render_pass: &mut wgpu::RenderPass, range: InstanceRange) {
        if range.start >= range.end || range.start >= self.instances.len() {
            return;
        }

        self.set_pipeline_state(render_pass);

        let real_end = range.end.min(self.instances.len());
        render_pass.draw(0..4, range.start as u32..real_end as u32);
    }

    pub fn prepare_decorations(&mut self) {
        self.text.prepare_decorations(&mut self.text_renderer);
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

        #[cfg(debug_assertions)]
        {
            if let Some(profiling_data) = self.gpu_profiler.process_finished_frame(self.queue.get_timestamp_period()) {
                for p in profiling_data {
                    if let Some(time) = p.time {
                        let dur = Duration::from_secs_f64(time.end - time.start);
                        println!("Gpu time ({}): {:?} s", p.label, dur);
                    }
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

#[cfg(test)]
mod tests {
    use crate::*;
    #[test]
    fn test_imported_shaders() {
        assert_imported_textslabs_shader_matches();
    }
}