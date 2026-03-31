pub mod shapes;
pub use shapes::*;
pub mod color;
pub use color::*;
pub mod gpu_vec;
pub mod images;

mod limited_hangout;
pub use limited_hangout::*;

use gpu_vec::GpuVec;
use std::hash::{Hash, Hasher};
use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub struct InstanceRange { pub start: usize, pub end: usize }

/// A range of instances in the deferred buffer.
/// Returned by `end_deferred_mode()` and used with `draw_deferred_elements()`.
#[derive(Debug, Clone, Copy)]
pub struct DeferredInstanceRange { start: usize, end: usize }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransformHandle(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClipRectHandle(usize);

pub use keru_text;

pub use keru_text::{
    Text, TextBoxHandle, TextEditHandle,
    TextStyle2, StyleHandle, ColorBrush, with_clipboard, BoundingBox,
    parley,
};
// Re-export font properties from parley
pub use keru_text::parley::{FontWeight, FontStyle, LineHeight};
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
    pub const QUADRATIC_BEZIER: u32 = 7;
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
    pub color_start: Color,
    pub color_end: Color,
    pub gradient_type: GradientType,
    pub angle: f32,
}

impl Gradient {
    pub const fn new(color_start: Color, color_end: Color) -> Self {
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

    pub const fn linear(color_start: Color, color_end: Color, angle: f32) -> Self {
        Self {
            color_start,
            color_end,
            gradient_type: GradientType::Linear,
            angle,
        }
    }

    pub const fn radial(color_start: Color, color_end: Color) -> Self {
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
        self.color_start.r.to_bits().hash(state);
        self.color_start.g.to_bits().hash(state);
        self.color_start.g.to_bits().hash(state);
        self.color_start.a.to_bits().hash(state);
        self.color_end.r.to_bits().hash(state);
        self.color_end.g.to_bits().hash(state);
        self.color_end.g.to_bits().hash(state);
        self.color_end.a.to_bits().hash(state);
        self.gradient_type.hash(state);
        self.angle.to_bits().hash(state);
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
    pub texture: Option<LoadedImage>,
}

/// Parameters for drawing a circle
#[derive(Debug, Clone)]
pub struct Circle {
    pub center: [f32; 2],
    pub radius: f32,
    pub fill: ColorFill,
    pub texture: Option<LoadedImage>,
}

/// Parameters for drawing a ring (hollow circle)
#[derive(Debug, Clone)]
pub struct CircleRing {
    pub center: [f32; 2],
    pub inner_radius: f32,
    pub outer_radius: f32,
    pub fill: ColorFill,
    pub texture: Option<LoadedImage>,
    pub dash_length: Option<f32>,
    pub dash_offset: f32,
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
    pub texture: Option<LoadedImage>,
    pub dash_length: Option<f32>,
    pub dash_offset: f32,
}

/// Parameters for drawing a pie slice
#[derive(Debug, Clone)]
pub struct CirclePie {
    pub center: [f32; 2],
    pub radius: f32,
    pub start_angle: f32,
    pub end_angle: f32,
    pub fill: ColorFill,
    pub texture: Option<LoadedImage>,
}

/// Parameters for drawing a line segment
#[derive(Debug, Clone)]
pub struct Segment {
    pub start: [f32; 2],
    pub end: [f32; 2],
    pub thickness: f32,
    pub fill: ColorFill,
    pub dash_length: Option<f32>,
    pub dash_offset: f32,
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
    pub color: Color,
    pub grid_type: GridType,
    pub texture: Option<LoadedImage>,
}

/// Parameters for drawing a triangle
#[derive(Debug, Clone)]
pub struct Triangle {
    pub p0: [f32; 2],
    pub p1: [f32; 2],
    pub p2: [f32; 2],
    pub fill: ColorFill,
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
    pub texture: Option<LoadedImage>,
}

/// Parameters for drawing a quadratic bezier curve from `p0` to `p2`, with `p1` as a control point.
///
/// The curve is rendered analytically, solving the cubic distance equation in the fragment shader. This is not cheap, relatively speaking.
#[derive(Debug, Clone)]
pub struct QuadraticBezier {
    pub p0: [f32; 2],
    pub p1: [f32; 2],
    pub p2: [f32; 2],
    pub thickness: f32,
    pub color: Color,
}

/// Parameters for drawing a dashed box outline (composed of segments and corner arcs)
#[derive(Debug, Clone)]
pub struct DashedBoxOutline {
    pub top_left: [f32; 2],
    pub size: [f32; 2],
    pub corner_radius: f32,
    pub thickness: f32,
    pub color: Color,
    pub dash_length: f32,
}

/// Parameters for drawing a dashed hexagon outline (composed of segments)
#[derive(Debug, Clone)]
pub struct DashedHexagonOutline {
    pub center: [f32; 2],
    pub size: f32,              // distance from center to vertex
    pub rotation: f32,          // rotation in radians (0 = flat-top)
    pub thickness: f32,
    pub color: Color,
    pub dash_length: f32,
}

fn fill_gpu(fill: ColorFill) -> ([f32; 2], Color, Color, u32) {
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

/// A clip rect
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ClipRect {
    pub x_clip: [f32; 2],
    pub y_clip: [f32; 2],
}
pub const CLIP_NOTHING: ClipRect = ClipRect {
    x_clip: [f32::MIN, f32::MAX],
    y_clip: [f32::MIN, f32::MAX],
};


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

pub type ClipRectOrTransform = [f32; 4];
impl From<Transform> for ClipRectOrTransform {
    fn from(t: Transform) -> Self {
        [t.offset[0], t.offset[1], t.scale, t._padding]
    }
}
impl From<ClipRectOrTransform> for Transform {
    fn from(s: ClipRectOrTransform) -> Self {
        Self {
            offset: [s[0], s[1]],
            scale: s[2],
            _padding: s[3],
        }
    }
}
impl From<ClipRect> for ClipRectOrTransform {
    fn from(c: ClipRect) -> Self {
        [c.x_clip[0], c.x_clip[1], c.y_clip[0], c.y_clip[1]]
    }
}
impl From<ClipRectOrTransform> for ClipRect {
    fn from(s: ClipRectOrTransform) -> Self {
        Self {
            x_clip: [s[0], s[1]],
            y_clip: [s[2], s[3]],
        }
    }
}

/// Combines a keru_draw Transform with a keru_text Transform2D.
fn combine_transforms(keru_transform: &Transform, keru_text_transform: &keru_text::Transform2D) -> keru_text::Transform2D {
    keru_text::Transform2D {
        translation: (
            keru_text_transform.translation.0 + keru_transform.offset[0],
            keru_text_transform.translation.1 + keru_transform.offset[1],
        ),
        rotation: keru_text_transform.rotation,
        scale: keru_text_transform.scale * keru_transform.scale,
    }
}

pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
    pub image_renderer: ImageRenderer,
    pub text: Text,
    shapes: Shapes,
    clip_rects_or_transforms: GpuVec<ClipRectOrTransform>,
    shapes_bind_group: wgpu::BindGroup,
    instances: GpuVec<Instance>,
    current_transform: usize,
    current_clip_rect: usize,
    pub gpu_profiler: GpuProfiler,
    // Deferred mode
    deferred_mode: bool,
    deferred_mode_start: usize,
    deferred_instances: Vec<Instance>,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Instance {
    p_type: u32,
    p_index: u32,
    transform_index: u32,
    clip_rect_index: u32,
}

impl Renderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        #[cfg(debug_assertions)] {
            assert_imported_keru_text_shader_matches();
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

        let text = Text::new(&device, &queue, surface_format);

        let shapes = Shapes::new(&device);
        let image_renderer = ImageRenderer::new(&device, &queue, surface_format);

        let mut clip_rects_or_transforms: GpuVec<ClipRectOrTransform> = GpuVec::new(&device, 64, "keru_draw clip_rects and transforms");
        clip_rects_or_transforms.push(Transform::identity().into());
        clip_rects_or_transforms.push(CLIP_NOTHING.into());

        // Create merged bind group layout for shapes + images
        let shapes_bind_group_layout = Self::create_shapes_bind_group_layout(&device);

        // Create merged bind group
        let shapes_bind_group = Self::create_shapes_bind_group(
            &device,
            &shapes_bind_group_layout,
            &clip_rects_or_transforms,
            &shapes,
            &image_renderer,
        );

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                // The binding indices has to match the order in which the parameter blocks appear in the shader!
                // If there are issues, compile the shaders with the -reflection-json flag and see the parameterBlock fields.
                // Binding order: shapes+images(0), keru_text(1)
                bind_group_layouts: &[
                    &shapes_bind_group_layout,
                    &text.bind_group_layout(),
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
                        wgpu::VertexAttribute {
                            offset: 12,
                            shader_location: 3,
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
            deferred_mode: false,
            deferred_mode_start: 0,
            deferred_instances: Vec::with_capacity(5),
            device: device.clone(),
            queue: queue.clone(),
            current_transform: 0, // Identity transform is at slot index 0
            current_clip_rect: 1, // "No clip" is at slot index 1
            render_pipeline, shapes, clip_rects_or_transforms, image_renderer, text, shapes_bind_group, instances, gpu_profiler
        }
    }

    fn create_shapes_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        let entries = &[
            GpuVec::<Transform>::bind_group_layout_entry(0),
            GpuVec::<BoxGpu>::bind_group_layout_entry(1),
            GpuVec::<CircleGpu>::bind_group_layout_entry(2),
            GpuVec::<SegmentGpu>::bind_group_layout_entry(3),
            GpuVec::<GridGpu>::bind_group_layout_entry(4),
            GpuVec::<TriangleGpu>::bind_group_layout_entry(5),
            GpuVec::<HexagonGpu>::bind_group_layout_entry(6),
            GpuVec::<QuadraticBezierGpu>::bind_group_layout_entry(7),
            // Texture atlas
            wgpu::BindGroupLayoutEntry {
                binding: 8,
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
                binding: 9,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ];

        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("keru_draw shapes+images bind group layout"),
            entries: entries,
        })
    }

    fn create_shapes_bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        clip_rects_or_transforms: &GpuVec<ClipRectOrTransform>,
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
                clip_rects_or_transforms.bind_group_entry(0),
                shapes.boxes.bind_group_entry(1),
                shapes.circles.bind_group_entry(2),
                shapes.segments.bind_group_entry(3),
                shapes.grids.bind_group_entry(4),
                shapes.triangles.bind_group_entry(5),
                shapes.hexagons.bind_group_entry(6),
                shapes.quadratic_beziers.bind_group_entry(7),
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 9,
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
            ..Default::default()
        });
        self.push_instance(Instance {
            p_type: primitive::BOX,
            p_index: index as u32,
            transform_index: self.current_transform as u32,
            clip_rect_index: self.current_clip_rect as u32,
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
    ) {
        self.draw_box(Box {
            top_left: [x, y],
            size: [width, height],
            corner_radius: 0.0,
            rounded_corners: RoundedCorners::NONE,
            border_thickness: 0.0,
            fill: ColorFill::Color(Color::WHITE),
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
            gradient_direction,
            color_start,
            color_end,
            gradient_type,
            texture_page,
            texture_uv_origin,
            texture_uv_size,
            dash_length: 0.0,
            dash_offset: 0.0,
        });
        self.push_instance(Instance {
            p_type: primitive::CIRCLE,
            p_index: index as u32,
            transform_index: self.current_transform as u32,
            clip_rect_index: self.current_clip_rect as u32,
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
            gradient_direction,
            color_start,
            color_end,
            gradient_type,
            texture_page,
            texture_uv_origin,
            texture_uv_size,
            dash_length: params.dash_length.unwrap_or(0.0),
            dash_offset: params.dash_offset,
        });
        self.push_instance(Instance {
            p_type: primitive::CIRCLE,
            p_index: index as u32,
            transform_index: self.current_transform as u32,
            clip_rect_index: self.current_clip_rect as u32,
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
            gradient_direction,
            color_start,
            color_end,
            gradient_type,
            texture_page,
            texture_uv_origin,
            texture_uv_size,
            dash_length: params.dash_length.unwrap_or(0.0),
            dash_offset: params.dash_offset,
        });
        self.push_instance(Instance {
            p_type: primitive::CIRCLE,
            p_index: index as u32,
            transform_index: self.current_transform as u32,
            clip_rect_index: self.current_clip_rect as u32,
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
            gradient_direction,
            color_start,
            color_end,
            gradient_type,
            texture_page,
            texture_uv_origin,
            texture_uv_size,
            dash_length: 0.0,
            dash_offset: 0.0,
        });
        self.push_instance(Instance {
            p_type: primitive::CIRCLE,
            p_index: index as u32,
            transform_index: self.current_transform as u32,
            clip_rect_index: self.current_clip_rect as u32,
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
            color_start,
            color_end,
            thickness_dash: [params.thickness, params.dash_length.unwrap_or(0.0), params.dash_offset, 0.0],
            gradient_type,
            texture_page,
            texture_uv_origin,
            texture_uv_size,
            pad: [0.0; 2],
        });
        self.push_instance(Instance {
            p_type: primitive::SEGMENT,
            p_index: index as u32,
            transform_index: self.current_transform as u32,
            clip_rect_index: self.current_clip_rect as u32,
        });
    }

    pub fn draw_grid(&mut self, params: Grid) {
        let (texture_uv_origin, texture_uv_size, texture_page) = texture_gpu(params.texture);

        let index = self.shapes.grids.len();
        self.shapes.grids.push(shapes::GridGpu {
            top_left: params.top_left,
            size: params.size,
            offset: params.offset,
            lattice_size: params.lattice_size,
            line_thickness: params.line_thickness,
            color: params.color,
            grid_type: params.grid_type as u32,
            texture_page,
            texture_uv_origin,
            texture_uv_size,
            pad: [0.0; 2],
        });
        self.push_instance(Instance {
            p_type: primitive::GRID,
            p_index: index as u32,
            transform_index: self.current_transform as u32,
            clip_rect_index: self.current_clip_rect as u32,
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
            gradient_direction,
            color_start,
            color_end,
            gradient_type,
            texture_page,
            texture_uv_origin,
            texture_uv_size,
            pad: [0.0; 2],
        });
        self.push_instance(Instance {
            p_type: primitive::TRIANGLE,
            p_index: index as u32,
            transform_index: self.current_transform as u32,
            clip_rect_index: self.current_clip_rect as u32,
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
            gradient_direction,
            stroke_thickness: params.stroke_thickness,
            texture_page,
            color_start,
            color_end,
            gradient_type,
            _pad1: 0.0,
            texture_uv_origin,
            texture_uv_size,
            pad: [0.0; 2],
        });
        self.push_instance(Instance {
            p_type: primitive::HEXAGON,
            p_index: index as u32,
            transform_index: self.current_transform as u32,
            clip_rect_index: self.current_clip_rect as u32,
        });
    }

    pub fn draw_quadratic_bezier(&mut self, params: QuadraticBezier) {
        let index = self.shapes.quadratic_beziers.len();
        self.shapes.quadratic_beziers.push(shapes::QuadraticBezierGpu {
            p0: params.p0,
            p1: params.p1,
            p2: params.p2,
            thickness: params.thickness,
            _pad0: 0.0,
            color: params.color,
        });
        self.push_instance(Instance {
            p_type: primitive::QUADRATIC_BEZIER,
            p_index: index as u32,
            transform_index: self.current_transform as u32,
            clip_rect_index: self.current_clip_rect as u32,
        });
    }

    /// Draw a dashed box outline using segments and corner arcs.
    pub fn draw_dashed_box_outline(&mut self, params: DashedBoxOutline) {
        let [x, y] = params.top_left;
        let [w, h] = params.size;
        let r = params.corner_radius;
        let fill = ColorFill::Color(params.color);
        let mut offset = 0.0f32;

        if r < 0.001 {
            // No rounded corners - just 4 segments
            // Top edge
            self.draw_segment(Segment {
                start: [x, y],
                end: [x + w, y],
                thickness: params.thickness,
                fill,
                dash_length: Some(params.dash_length),
                dash_offset: offset,
                texture: None,
            });
            offset += w;
            // Right edge
            self.draw_segment(Segment {
                start: [x + w, y],
                end: [x + w, y + h],
                thickness: params.thickness,
                fill,
                dash_length: Some(params.dash_length),
                dash_offset: offset,
                texture: None,
            });
            offset += h;
            // Bottom edge
            self.draw_segment(Segment {
                start: [x + w, y + h],
                end: [x, y + h],
                thickness: params.thickness,
                fill,
                dash_length: Some(params.dash_length),
                dash_offset: offset,
                texture: None,
            });
            offset += w;
            // Left edge
            self.draw_segment(Segment {
                start: [x, y + h],
                end: [x, y],
                thickness: params.thickness,
                fill,
                dash_length: Some(params.dash_length),
                dash_offset: offset,
                texture: None,
            });
        } else {
            // Rounded corners - 4 segments + 4 quarter arcs
            let pi = std::f32::consts::PI;
            let quarter_arc = r * pi * 0.5; // length of quarter circle arc

            // Top edge (between top-left and top-right corners)
            let top_len = w - 2.0 * r;
            self.draw_segment(Segment {
                start: [x + r, y],
                end: [x + w - r, y],
                thickness: params.thickness,
                fill,
                dash_length: Some(params.dash_length),
                dash_offset: offset,
                texture: None,
            });
            offset += top_len;
            // Top-right corner arc
            self.draw_arc(CircleArc {
                center: [x + w - r, y + r],
                radius: r,
                start_angle: -pi * 0.5,
                end_angle: 0.0,
                thickness: params.thickness,
                fill,
                texture: None,
                dash_length: Some(params.dash_length),
                dash_offset: offset,
            });
            offset += quarter_arc;
            // Right edge
            let right_len = h - 2.0 * r;
            self.draw_segment(Segment {
                start: [x + w, y + r],
                end: [x + w, y + h - r],
                thickness: params.thickness,
                fill,
                dash_length: Some(params.dash_length),
                dash_offset: offset,
                texture: None,
            });
            offset += right_len;
            // Bottom-right corner arc
            self.draw_arc(CircleArc {
                center: [x + w - r, y + h - r],
                radius: r,
                start_angle: 0.0,
                end_angle: pi * 0.5,
                thickness: params.thickness,
                fill,
                texture: None,
                dash_length: Some(params.dash_length),
                dash_offset: offset,
            });
            offset += quarter_arc;
            // Bottom edge
            self.draw_segment(Segment {
                start: [x + w - r, y + h],
                end: [x + r, y + h],
                thickness: params.thickness,
                fill,
                dash_length: Some(params.dash_length),
                dash_offset: offset,
                texture: None,
            });
            offset += top_len;
            // Bottom-left corner arc
            self.draw_arc(CircleArc {
                center: [x + r, y + h - r],
                radius: r,
                start_angle: pi * 0.5,
                end_angle: pi,
                thickness: params.thickness,
                fill,
                texture: None,
                dash_length: Some(params.dash_length),
                dash_offset: offset,
            });
            offset += quarter_arc;
            // Left edge
            self.draw_segment(Segment {
                start: [x, y + h - r],
                end: [x, y + r],
                thickness: params.thickness,
                fill,
                dash_length: Some(params.dash_length),
                dash_offset: offset,
                texture: None,
            });
            offset += right_len;
            // Top-left corner arc
            self.draw_arc(CircleArc {
                center: [x + r, y + r],
                radius: r,
                start_angle: pi,
                end_angle: pi * 1.5,
                thickness: params.thickness,
                fill,
                texture: None,
                dash_length: Some(params.dash_length),
                dash_offset: offset,
            });
        }
    }

    /// Draw a dashed hexagon outline using 6 segments.
    pub fn draw_dashed_hexagon_outline(&mut self, params: DashedHexagonOutline) {
        let fill = ColorFill::Color(params.color);
        let pi = std::f32::consts::PI;

        // Calculate the 6 vertices of the hexagon
        let mut vertices = [[0.0f32; 2]; 6];
        for i in 0..6 {
            let angle = params.rotation + (i as f32) * pi / 3.0;
            vertices[i] = [
                params.center[0] + params.size * angle.cos(),
                params.center[1] + params.size * angle.sin(),
            ];
        }

        // Edge length of regular hexagon = size (distance from center to vertex)
        let edge_len = params.size;
        let mut offset = 0.0f32;

        // Draw 6 segments connecting the vertices
        for i in 0..6 {
            let next = (i + 1) % 6;
            self.draw_segment(Segment {
                start: vertices[i],
                end: vertices[next],
                thickness: params.thickness,
                fill,
                dash_length: Some(params.dash_length),
                dash_offset: offset,
                texture: None,
            });
            offset += edge_len;
        }
    }

    /// Draw a text box.
    pub fn draw_text_box(&mut self, text_box: &TextBoxHandle) {
        // Get current transform before borrowing text_box_ref
        let current_euclid_transform = self.get_current_transform();

        // Combine keru_draw's transform with the text box's retained_transform
        let text_box_ref = self.text.get_text_box_mut(text_box);
        let retained = text_box_ref.transform();

        // Combine: first apply retained_transform, then keru_draw's transform
        let combined = combine_transforms(&current_euclid_transform, &retained);
        text_box_ref.set_transform(combined);

        let glyph_range = self.text.get_text_box(text_box).glyph_quad_range();

        for q in (glyph_range.0)..(glyph_range.1) {
            self.push_instance(Instance {
                p_type: primitive::TEXT,
                p_index: q as u32,
                transform_index: self.current_transform as u32,
                clip_rect_index: self.current_clip_rect as u32,
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

        let glyph_range = self.text.get_text_edit(text_edit).glyph_quad_range();

        for q in (glyph_range.0)..(glyph_range.1) {
            self.push_instance(Instance {
                p_type: primitive::TEXT,
                p_index: q as u32,
                transform_index: self.current_transform as u32,
                clip_rect_index: self.current_clip_rect as u32,
            });
        }
    }

    /// Begin recording a new frame.
    ///
    /// This only clears the main instance buffer, not the shape data, transforms, clip_rects, or deferred instances, so they can be reused.
    ///
    /// To clear everything, call [`Renderer::clear_for_new_frame()`].
    pub fn begin_frame(&mut self) {
        self.instances.clear();
        self.current_transform = 0; // Reset to identity transform
        self.current_clip_rect = 1; // Reset to "no clip"
        self.deferred_mode = false;
        self.deferred_mode_start = 0;
    }

    /// Clear all the render data, including shapes, deferred instances, transforms, and clip_rects, and begin a new frame from scratch.
    pub fn clear_for_new_frame(&mut self) {
        self.instances.clear();
        self.shapes.clear();
        self.deferred_instances.clear();
        // Reset slots: identity transform at index 0, "no clip" at index 1
        self.clip_rects_or_transforms.clear();
        self.clip_rects_or_transforms.push(Transform::identity().into());
        self.clip_rects_or_transforms.push(CLIP_NOTHING.into());
        self.current_transform = 0;
        self.current_clip_rect = 1;
    }

    pub fn prepare_text(&mut self) {
        self.text.prepare_all();
    }

    /// Internal helper to push an instance to the correct buffer based on deferred mode.
    fn push_instance(&mut self, instance: Instance) {
        if self.deferred_mode {
            self.deferred_instances.push(instance);
        } else {
            self.instances.push(instance);
        }
    }

    /// Start deferred mode. While in deferred mode, draw calls will be recorded
    /// but not added to the main instance buffer. Call `end_deferred_mode()` to
    /// get a handle to the recorded instances, which can later be drawn with
    /// `draw_deferred_elements()`.
    pub fn start_deferred_mode(&mut self) {
        self.deferred_mode = true;
        self.deferred_mode_start = self.deferred_instances.len();
    }

    /// End deferred mode and return a handle to the recorded instances.
    /// The returned [`DeferredInstanceRange`] can be used with `draw_deferred_elements()`to copy the recorded instances to the main buffer at any time.
    pub fn end_deferred_mode(&mut self) -> DeferredInstanceRange {
        self.deferred_mode = false;
        DeferredInstanceRange {
            start: self.deferred_mode_start,
            end: self.deferred_instances.len(),
        }
    }

    /// Copy deferred instances to the main instance buffer.
    /// 
    /// This allows drawing primitives in a different order than they were created.
    pub fn draw_deferred_elements(&mut self, range: DeferredInstanceRange) {
        self.instances.vec_mut().extend_from_slice(&self.deferred_instances[range.start..range.end]);                                                         
    }

    /// Set the current transform to an existing transform handle.
    /// All subsequent draw calls will use this transform until `pop_current_transform` is called.
    pub fn set_current_transform(&mut self, handle: TransformHandle) {
        self.current_transform = handle.0;
    }

    /// Reset the current transform back to identity.
    pub fn clear_current_transform(&mut self) {
        self.current_transform = 0;
    }

    /// Create a transform for this frame.
    /// The returned `TransformHandle` is valid until the next time [`Renderer::clear_for_new_frame()`] is called.
    pub fn insert_transform(&mut self, transform: Transform) -> TransformHandle {
        let index = self.clip_rects_or_transforms.len();
        self.clip_rects_or_transforms.push(transform.into());
        TransformHandle(index)
    }

    /// Modify a transform.
    /// All instances using this transform will be affected.
    pub fn update_transform(&mut self, handle: TransformHandle, transform: Transform) {
        self.clip_rects_or_transforms[handle.0] = transform.into()
    }

    /// Get the value of a transform.
    pub fn get_transform(&self, handle: TransformHandle) -> Transform {
        self.clip_rects_or_transforms[handle.0].into()
    }

    /// Set the current clip rect to an existing clip rect handle.
    /// All subsequent draw calls will use this clip rect until `clear_current_clip_rect` is called.
    pub fn set_current_clip_rect(&mut self, handle: ClipRectHandle) {
        self.current_clip_rect = handle.0;
    }

    /// Reset the current clip rect back to "no clip".
    pub fn clear_current_clip_rect(&mut self) {
        self.current_clip_rect = 0;
    }

    /// Create a clip rect for this frame.
    /// The returned `ClipRectHandle` is valid until the next time [`Renderer::clear_for_new_frame()`] is called.
    pub fn insert_clip_rect(&mut self, clip_rect: ClipRect) -> ClipRectHandle {
        let index = self.clip_rects_or_transforms.len();
        self.clip_rects_or_transforms.push(clip_rect.into());
        ClipRectHandle(index)
    }

    /// Modify a clip rect.
    /// All instances using this clip rect will be affected.
    pub fn update_clip_rect(&mut self, handle: ClipRectHandle, clip_rect: ClipRect) {
        self.clip_rects_or_transforms[handle.0] = clip_rect.into();
    }

    /// Get the value of a clip rect.
    pub fn get_clip_rect(&self, handle: ClipRectHandle) -> ClipRect {
        self.clip_rects_or_transforms[handle.0].into()
    }

    /// Get the "no clip" handle (index 0).
    pub fn no_clip(&self) -> ClipRectHandle {
        ClipRectHandle(0)
    }


    /// Get the current transform being used for draw calls.
    fn get_current_transform(&self) -> Transform {
        let current_index = self.current_transform;
        if current_index < self.clip_rects_or_transforms.len() {
            self.clip_rects_or_transforms[current_index].into()
        } else {
            Transform::identity()
        }
    }

    /// Render into a render pass.
    pub fn render(&mut self, render_pass: &mut wgpu::RenderPass) {
        // Upload resources to GPU
        let slots_changed = self.clip_rects_or_transforms.load_to_gpu(&self.device, &self.queue);
        let shapes_changed = self.shapes.load_to_gpu(&self.device, &self.queue);
        let images_changed = self.image_renderer.load_to_gpu(&self.device, &self.queue);

        // Recreate bind group if slots, shapes or images changed
        if slots_changed || shapes_changed || images_changed {
            let layout = Self::create_shapes_bind_group_layout(&self.device);
            self.shapes_bind_group = Self::create_shapes_bind_group(
                &self.device,
                &layout,
                &self.clip_rects_or_transforms,
                &self.shapes,
                &self.image_renderer,
            );
        }

        self.text.load_to_gpu();
        self.instances.load_to_gpu(&self.device, &self.queue);

        self.set_pipeline_state(render_pass);

        render_pass.draw(0..4, 0..self.instances.len() as u32);
    }

    pub fn load_to_gpu(&mut self) {
        // Upload resources to GPU
        let slots_changed = self.clip_rects_or_transforms.load_to_gpu(&self.device, &self.queue);
        let shapes_changed = self.shapes.load_to_gpu(&self.device, &self.queue);
        let images_changed = self.image_renderer.load_to_gpu(&self.device, &self.queue);

        // Recreate bind group if slots, shapes or images changed
        if slots_changed || shapes_changed || images_changed {
            let layout = Self::create_shapes_bind_group_layout(&self.device);
            self.shapes_bind_group = Self::create_shapes_bind_group(
                &self.device,
                &layout,
                &self.clip_rects_or_transforms,
                &self.shapes,
                &self.image_renderer,
            );
        }

        self.text.load_to_gpu();
        self.instances.load_to_gpu(&self.device, &self.queue);
    }

    pub fn set_pipeline_state(&mut self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_pipeline(&self.render_pipeline);
        // The binding indices has to match the order in which the parameter blocks appear in the shader!
        // If there are issues, compile the shaders with the -reflection-json flag and see the parameterBlock fields.
        // Binding order: shapes+images(0), keru_text(1)
        render_pass.set_bind_group(0, &self.shapes_bind_group, &[]);
        render_pass.set_bind_group(1, &self.text.bind_group(), &[]);
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
            let profiling_data = self.gpu_profiler.process_finished_frame(self.queue.get_timestamp_period());
            if let Some(profiling_data) = profiling_data {
                for p in profiling_data {
                    if let Some(time) = p.time {
                        let dur = Duration::from_secs_f64(time.end - time.start);
                        println!("Gpu time ({}): {:?}", p.label, dur);
                    }
                }
            }
        }

        output.present();
    }

    // todo: remove this and make nicer start/end methods 
    pub fn instance_count(&self) -> usize {
        self.instances.len()
    }
}

fn assert_imported_keru_text_shader_matches() {
    let imported_shader = include_str!("shaders/keru_text.slang");
    let original_shader = keru_text::Text::composable_shader_source();
    assert!(imported_shader == original_shader);
}

#[cfg(test)]
mod tests {
    use crate::*;
    #[test]
    fn test_imported_shaders() {
        assert_imported_keru_text_shader_matches();
    }
}